use std::fs;
use std::os::fd::OwnedFd;

use rustix::io::Errno;
use rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal};

use crate::domain::process::parse_stat;
use crate::ports::{Claim, Identity, SignalError, Signals, Stop};

pub struct ProcessSignals;

impl Signals for ProcessSignals {
    fn claim(&self, identity: Identity) -> Result<Box<dyn Claim>, SignalError> {
        let pid = i32::try_from(identity.pid)
            .ok()
            .and_then(Pid::from_raw)
            .ok_or(SignalError::Gone)?;
        let pidfd = pidfd_open(pid, PidfdFlags::empty()).map_err(failure)?;
        let started = fs::read_to_string(format!("/proc/{}/stat", identity.pid))
            .ok()
            .and_then(|text| parse_stat(&text))
            .map(|stat| stat.started);
        if started != Some(identity.started) {
            return Err(SignalError::Gone);
        }
        Ok(Box::new(PidFd(pidfd)))
    }
}

struct PidFd(OwnedFd);

impl Claim for PidFd {
    fn send(&self, stop: Stop) -> Result<(), SignalError> {
        let signal = match stop {
            Stop::Terminate => Signal::TERM,
            Stop::Kill => Signal::KILL,
        };
        pidfd_send_signal(&self.0, signal).map_err(failure)
    }
}

fn failure(errno: Errno) -> SignalError {
    match errno {
        Errno::PERM => SignalError::Denied,
        Errno::SRCH => SignalError::Gone,
        other => SignalError::Failed(other.to_string()),
    }
}
