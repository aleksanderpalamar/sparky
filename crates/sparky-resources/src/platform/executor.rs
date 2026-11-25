use std::io;
use std::path::Path;

use sparky_hardware::process::parse_stat;

use super::syscalls;
use crate::domain::{Change, ChangeKind, Identity, IoClass};
use crate::ports::{Elevator, ExecError, Executor, Files};

const MAX_CPUS: u32 = 1024;

pub struct LinuxExecutor<F, E> {
    files: F,
    elevator: E,
}

impl<F: Files, E: Elevator> LinuxExecutor<F, E> {
    pub fn new(files: F, elevator: E) -> Self {
        Self { files, elevator }
    }

    fn renice(&self, task: u32, value: i8) -> Result<(), ExecError> {
        match syscalls::set_nice(task, value).map_err(failure) {
            Err(ExecError::Denied) => self.elevator.renice(task, value),
            result => result,
        }
    }
}

impl<F: Files, E: Elevator> Executor for LinuxExecutor<F, E> {
    fn tasks(&self, target: Identity) -> Result<Vec<u32>, ExecError> {
        let stat = self
            .files
            .read(Path::new(&format!("/proc/{}/stat", target.pid)))
            .map_err(unreadable)?;
        let started = parse_stat(&stat)
            .map(|stat| stat.started)
            .ok_or(ExecError::Failed)?;
        if started != target.started {
            return Err(ExecError::Gone);
        }
        let entries = self
            .files
            .entries(Path::new(&format!("/proc/{}/task", target.pid)))
            .map_err(unreadable)?;
        Ok(entries
            .iter()
            .filter_map(|name| name.parse().ok())
            .collect())
    }

    fn read(&self, task: u32, kind: ChangeKind) -> Result<Change, ExecError> {
        match kind {
            ChangeKind::Nice => syscalls::nice(task).map(Change::Nice).map_err(failure),
            ChangeKind::IoPriority => {
                io_class(syscalls::io_priority(task).map_err(failure)?).map(Change::IoPriority)
            }
            ChangeKind::Affinity => syscalls::affinity(task)
                .map(Change::Affinity)
                .map_err(failure),
        }
    }

    fn write(&self, task: u32, change: &Change) -> Result<(), ExecError> {
        let result = match change {
            Change::Nice(value) => return self.renice(task, *value),
            Change::IoPriority(IoClass::None) => syscalls::set_io_priority(task, 0, 0),
            Change::IoPriority(IoClass::BestEffort(level)) => {
                syscalls::set_io_priority(task, 2, *level)
            }
            Change::IoPriority(IoClass::Idle) => syscalls::set_io_priority(task, 3, 0),
            Change::Affinity(cpus) if cpus.iter().all(|cpu| *cpu < MAX_CPUS) => {
                syscalls::set_affinity(task, cpus)
            }
            Change::Affinity(_) => return Err(ExecError::Unsupported),
        };
        result.map_err(failure)
    }
}

fn io_class((class, data): (u8, u8)) -> Result<IoClass, ExecError> {
    match class {
        0 => Ok(IoClass::None),
        2 => Ok(IoClass::BestEffort(data)),
        3 => Ok(IoClass::Idle),
        _ => Err(ExecError::Unsupported),
    }
}

fn unreadable(error: io::Error) -> ExecError {
    match error.kind() {
        io::ErrorKind::NotFound => ExecError::Gone,
        io::ErrorKind::PermissionDenied => ExecError::Denied,
        _ => failure(error),
    }
}

fn failure(error: io::Error) -> ExecError {
    match error.raw_os_error() {
        Some(libc::ESRCH) => ExecError::Gone,
        Some(libc::EPERM | libc::EACCES) => ExecError::Denied,
        _ => ExecError::Failed,
    }
}
