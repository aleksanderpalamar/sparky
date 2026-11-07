use std::env;
use std::ffi::CString;
use std::fs;
use std::io;
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::linux::command;
use crate::linux::host::{Host, Ran, Usage};

pub struct System;

impl Host for System {
    fn read(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn list(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect()
    }

    fn var(&self, name: &str) -> Option<String> {
        env::var(name).ok()
    }

    fn usage(&self, path: &Path) -> io::Result<Usage> {
        let path = CString::new(path.as_os_str().as_bytes()).map_err(io::Error::other)?;
        let mut stats = MaybeUninit::<libc::statvfs>::zeroed();
        let result = unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        let stats = unsafe { stats.assume_init() };
        Ok(Usage {
            total: bytes(stats.f_blocks, stats.f_frsize),
            free: bytes(stats.f_bfree, stats.f_frsize),
        })
    }

    fn run(&self, program: &str, arguments: &[&str], timeout: Duration) -> Ran {
        command::run(program, arguments, timeout)
    }
}

fn bytes(blocks: impl Into<u64>, size: impl Into<u64>) -> u64 {
    blocks.into().saturating_mul(size.into())
}
