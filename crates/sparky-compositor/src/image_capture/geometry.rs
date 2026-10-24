use super::types::{SessionData, Target};
use crate::state::Sparky;
use smithay::reexports::wayland_server::Resource;
use smithay::utils::{Physical, Size};

impl Sparky {
    pub fn refresh_capture_sessions(&mut self) {
        self.image_capture
            .sessions
            .retain(|weak| weak.upgrade().is_ok());
        for weak in &self.image_capture.sessions {
            let Ok(session) = weak.upgrade() else {
                continue;
            };
            let Some(data) = session.data::<std::sync::Arc<SessionData>>() else {
                continue;
            };
            let Ok(mut status) = data.status.lock() else {
                continue;
            };
            if status.stopped {
                continue;
            }
            let Some(size) = self.capture_dimensions(&data.target) else {
                status.stopped = true;
                session.stopped();
                continue;
            };
            if status.size == size {
                continue;
            }
            status.size = size;
            self.image_capture.formats.announce(&session, size);
        }
    }

    pub fn capture_dimensions(&self, target: &Target) -> Option<(i32, i32)> {
        let size: Size<i32, Physical> = match target {
            Target::Output(weak) => {
                let output = weak.upgrade()?;
                let size = self.space.output_geometry(&output)?.size;
                size.to_physical_precise_round(output.current_scale().fractional_scale())
            }
            Target::Window(id) => {
                let window = self.windows.get(*id)?;
                let scale = self
                    .space
                    .outputs_for_element(window)
                    .first()
                    .map_or(1.0, |out| out.current_scale().fractional_scale());
                window.geometry().size.to_physical_precise_round(scale)
            }
            Target::Unavailable => return None,
        };
        (size.w > 0 && size.h > 0 && size.w <= 16384 && size.h <= 16384).then_some((size.w, size.h))
    }
}
