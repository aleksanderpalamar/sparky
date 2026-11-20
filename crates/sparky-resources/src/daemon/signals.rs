use nix::sys::signal::{SigSet, Signal};

pub struct Stop(SigSet);

impl Stop {
    pub fn block() -> nix::Result<Self> {
        let mut set = SigSet::empty();
        for signal in [Signal::SIGTERM, Signal::SIGINT, Signal::SIGHUP] {
            set.add(signal);
        }
        set.thread_block()?;
        Ok(Self(set))
    }

    pub fn wait(&self) -> nix::Result<Signal> {
        self.0.wait()
    }
}
