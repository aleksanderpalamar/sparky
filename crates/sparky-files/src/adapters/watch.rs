use std::io;
use std::mem::MaybeUninit;
use std::os::fd::{AsFd, OwnedFd};
use std::path::Path;
use std::time::{Duration, Instant};

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::fs::inotify::{self, CreateFlags, ReadFlags, WatchFlags};

use crate::domain::debounce::Debounce;

const WINDOW: Duration = Duration::from_millis(400);
const TICK: Timespec = Timespec {
    tv_sec: 0,
    tv_nsec: 200_000_000,
};

pub fn watch(
    dir: &Path,
    alive: impl Fn() -> bool,
    mut changed: impl FnMut() -> bool,
) -> io::Result<()> {
    let fd = inotify::init(CreateFlags::CLOEXEC | CreateFlags::NONBLOCK)?;
    inotify::add_watch(&fd, dir, interesting())?;
    let mut debounce = Debounce::new(WINDOW);
    while alive() {
        if wait(&fd)? && drain(&fd)? {
            debounce.note(Instant::now());
        }
        if debounce.fire(Instant::now()) && !changed() {
            return Ok(());
        }
    }
    Ok(())
}

fn interesting() -> WatchFlags {
    WatchFlags::CREATE
        | WatchFlags::DELETE
        | WatchFlags::MOVED_FROM
        | WatchFlags::MOVED_TO
        | WatchFlags::CLOSE_WRITE
        | WatchFlags::ATTRIB
        | WatchFlags::DELETE_SELF
        | WatchFlags::MOVE_SELF
}

fn wait(fd: &OwnedFd) -> io::Result<bool> {
    let mut fds = [PollFd::new(fd, PollFlags::IN)];
    match poll(&mut fds, Some(&TICK)) {
        Ok(ready) => Ok(ready > 0),
        Err(rustix::io::Errno::INTR) => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn drain(fd: &OwnedFd) -> io::Result<bool> {
    let mut buffer = [MaybeUninit::<u8>::uninit(); 4096];
    let mut reader = inotify::Reader::new(fd.as_fd(), &mut buffer);
    let mut relevant = false;
    loop {
        match reader.next() {
            Ok(event) => relevant |= !event.events().contains(ReadFlags::IGNORED),
            Err(rustix::io::Errno::AGAIN) => return Ok(relevant),
            Err(rustix::io::Errno::INTR) => continue,
            Err(error) => return Err(error.into()),
        }
    }
}
