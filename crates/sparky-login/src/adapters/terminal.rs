use std::fs::OpenOptions;
use std::io;
use std::os::fd::AsRawFd;

use crate::domain::environment::tty;

const VT_ACTIVATE: libc::c_ulong = 0x5606;
const VT_WAITACTIVE: libc::c_ulong = 0x5607;

pub fn activate(vt: u32) -> io::Result<()> {
    let console = OpenOptions::new().read(true).write(true).open(tty(vt))?;
    let number = libc::c_int::try_from(vt).map_err(io::Error::other)?;
    for request in [VT_ACTIVATE, VT_WAITACTIVE] {
        if unsafe { libc::ioctl(console.as_raw_fd(), request, number) } != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}
