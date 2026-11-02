use std::ffi::CString;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use nix::sys::signal::{Signal, kill};
use nix::unistd::{Gid, Pid, Uid, chdir, getgrouplist, setgid, setgroups, setsid, setuid};

use crate::adapters::stop;
use crate::domain::environment::Account;
use crate::ports::launcher::{Launcher, Process};

const POLL: Duration = Duration::from_millis(100);
const ROOT: &str = "/";

pub struct UserLauncher;

struct Running(Child);

struct Identity {
    uid: Uid,
    gid: Gid,
    groups: Vec<Gid>,
    home: CString,
    root: CString,
}

impl Launcher for UserLauncher {
    fn launch(
        &self,
        account: &Account,
        command: &[String],
        environment: &[(String, String)],
        tty: &Path,
    ) -> Result<Box<dyn Process>, String> {
        let (program, arguments) = command.split_first().ok_or("comando vazio")?;
        let terminal = OpenOptions::new()
            .read(true)
            .write(true)
            .open(tty)
            .map_err(|error| format!("{}: {error}", tty.display()))?;
        let identity = identity(account)?;
        let mut process = Command::new(program);
        process
            .args(arguments)
            .env_clear()
            .envs(environment.iter().cloned());
        process
            .stdin(copy(&terminal)?)
            .stdout(copy(&terminal)?)
            .stderr(terminal);
        unsafe {
            process.pre_exec(move || become_user(&identity));
        }
        let child = process
            .spawn()
            .map_err(|error| format!("{program}: {error}"))?;
        Ok(Box::new(Running(child)))
    }
}

impl Process for Running {
    fn wait(&mut self) -> Result<(), String> {
        loop {
            if let Some(_status) = self.0.try_wait().map_err(|error| error.to_string())? {
                return Ok(());
            }
            if stop::requested() {
                let pid =
                    Pid::from_raw(i32::try_from(self.0.id()).map_err(|error| error.to_string())?);
                let _ = kill(pid, Signal::SIGTERM);
                return self.0.wait().map(drop).map_err(|error| error.to_string());
            }
            thread::sleep(POLL);
        }
    }
}

fn identity(account: &Account) -> Result<Identity, String> {
    let name = CString::new(account.name.as_str()).map_err(|error| error.to_string())?;
    let gid = Gid::from_raw(account.gid);
    let groups =
        getgrouplist(&name, gid).map_err(|error| format!("grupos de {}: {error}", account.name))?;
    let home = CString::new(account.home.to_string_lossy().as_bytes())
        .map_err(|error| error.to_string())?;
    let root = CString::new(ROOT).map_err(|error| error.to_string())?;
    Ok(Identity {
        uid: Uid::from_raw(account.uid),
        gid,
        groups,
        home,
        root,
    })
}

fn become_user(identity: &Identity) -> io::Result<()> {
    setsid()?;
    if unsafe { libc::ioctl(0, libc::TIOCSCTTY, 0) } != 0 {
        return Err(io::Error::last_os_error());
    }
    setgroups(&identity.groups)?;
    setgid(identity.gid)?;
    setuid(identity.uid)?;
    if chdir(identity.home.as_c_str()).is_err() {
        chdir(identity.root.as_c_str())?;
    }
    Ok(())
}

fn copy(file: &File) -> Result<File, String> {
    file.try_clone().map_err(|error| error.to_string())
}
