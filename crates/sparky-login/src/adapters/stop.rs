use std::sync::atomic::{AtomicBool, Ordering};

use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal, sigaction};

static REQUESTED: AtomicBool = AtomicBool::new(false);

extern "C" fn mark(_: libc::c_int) {
    REQUESTED.store(true, Ordering::SeqCst);
}

pub fn watch() -> Result<(), String> {
    let action = SigAction::new(SigHandler::Handler(mark), SaFlags::empty(), SigSet::empty());
    for signal in [Signal::SIGTERM, Signal::SIGINT, Signal::SIGHUP] {
        unsafe { sigaction(signal, &action) }
            .map_err(|error| format!("sinal {signal}: {error}"))?;
    }
    Ok(())
}

pub fn requested() -> bool {
    REQUESTED.load(Ordering::SeqCst)
}
