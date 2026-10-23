use std::time::Duration;

use smithay::reexports::calloop::timer::{TimeoutAction, Timer};

use crate::backend::Backend;
use crate::state::Sparky;

pub fn start(state: &Sparky) {
    let interval = Duration::from_secs(1);
    let result =
        state
            .loop_handle
            .insert_source(Timer::from_duration(interval), move |_, _, state| {
                let (outputs, pending, timers) = match &state.backend {
                    Backend::Drm(drm) => (
                        drm.gpu.surfaces.len(),
                        drm.gpu
                            .surfaces
                            .values()
                            .filter(|surface| surface.frame_pending)
                            .count(),
                        drm.gpu
                            .surfaces
                            .values()
                            .filter(|surface| surface.idle_timer.is_some())
                            .count(),
                    ),
                    Backend::Nested(_) => (1, 0, 0),
                    Backend::Starting => (0, 0, 0),
                };
                state.diagnostics.topology(
                    state.windows.iter().count() + state.unmanaged.len(),
                    outputs,
                    pending,
                    timers,
                );
                TimeoutAction::ToDuration(interval)
            });
    if let Err(error) = result {
        tracing::warn!("sem timer de diagnóstico: {error}");
    }
}
