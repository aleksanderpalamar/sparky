use calloop::signals::{Signal, Signals};
use smithay::reexports::calloop::LoopHandle;

use crate::state::Sparky;

const ENDING: [Signal; 3] = [Signal::SIGTERM, Signal::SIGINT, Signal::SIGHUP];

pub fn stop_on_signals(handle: &LoopHandle<'static, Sparky>) -> Result<(), Box<dyn std::error::Error>> {
    let signals = Signals::new(&ENDING)?;
    handle.insert_source(signals, |event, _, state| {
        tracing::info!("sparky encerrando por {:?}", event.signal());
        state.loop_signal.stop();
    })?;
    Ok(())
}
