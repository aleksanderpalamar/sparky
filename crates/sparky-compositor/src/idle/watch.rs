use std::time::Duration;

use smithay::reexports::calloop::timer::{TimeoutAction, Timer};

use crate::idle::rules::due;
use crate::state::Sparky;

const CHECK: Duration = Duration::from_secs(5);

impl Sparky {
    pub fn watch_idle(&mut self) {
        let watching =
            self.loop_handle
                .insert_source(Timer::from_duration(CHECK), |_, _, state| {
                    state.check_idle();
                    TimeoutAction::ToDuration(CHECK)
                });
        if let Err(error) = watching {
            tracing::warn!("sessão: sem bloqueio por inatividade: {error}");
        }
    }

    fn check_idle(&mut self) {
        let inhibition = self.idle.inhibition();
        let limit = self.config.lock.idle_limit();
        if !due(self.idle.idle_for(), limit, inhibition, self.lock.phase()) {
            return;
        }
        tracing::info!("sessão: bloqueio por inatividade");
        self.idle.restart();
        self.lock_screen();
    }
}
