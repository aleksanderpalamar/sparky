use rustix::io::Errno;
use rustix::process::{Pid, Signal, kill_process};

use crate::ports::{SignalError, Signals, Stop};

pub struct ProcessSignals;

impl Signals for ProcessSignals {
    fn send(&self, pid: u32, stop: Stop) -> Result<(), SignalError> {
        let pid = i32::try_from(pid)
            .ok()
            .and_then(Pid::from_raw)
            .ok_or(SignalError::Gone)?;
        let signal = match stop {
            Stop::Terminate => Signal::TERM,
            Stop::Kill => Signal::KILL,
        };
        kill_process(pid, signal).map_err(|errno| match errno {
            Errno::PERM => SignalError::Denied,
            Errno::SRCH => SignalError::Gone,
            other => SignalError::Failed(other.to_string()),
        })
    }
}
