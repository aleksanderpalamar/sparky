use std::time::Duration;

use smithay::reexports::calloop::timer::{TimeoutAction, Timer};

use crate::config::disk;
use crate::state::Sparky;

const INTERVAL: Duration = Duration::from_secs(1);

impl Sparky {
    pub fn watch_config(&mut self) {
        let watching = self.loop_handle.insert_source(Timer::from_duration(INTERVAL), |_, _, state| {
            if disk::stamp(&disk::path()) != state.config_stamp
                && let Err(error) = state.reload_config()
            {
                tracing::warn!("{error}; a configuração anterior continua valendo");
            }
            if state.colors.update(sparky_theme::current_palette()) {
                state.needs_render = true;
            }
            TimeoutAction::ToDuration(INTERVAL)
        });
        if let Err(error) = watching {
            tracing::warn!("a configuração não será recarregada sozinha: {error}");
        }
    }
}
