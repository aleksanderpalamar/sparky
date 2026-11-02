use std::sync::Arc;

use smithay::output::Output;
use smithay::reexports::wayland_server::Resource;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use super::presets::OutputColor;
use super::protocol::feedback::{self, FeedbackData};
use super::protocol::output;
use super::records::Record;
use super::state::Change;
use crate::state::Sparky;

impl Sparky {
    pub fn refresh_color_management(&mut self) {
        if let Some(colors) = self.output_colors() {
            self.apply_output_colors(colors);
        }
        self.refresh_preferred();
    }

    pub fn apply_output_colors(&mut self, colors: Vec<(Output, OutputColor)>) {
        for (monitor, color) in colors {
            if self.color_management.set(&monitor, color) == Change::Same {
                continue;
            }
            tracing::info!(output = %monitor.name(), ?color, "descrição de cor do monitor mudou");
            output::announce(&mut self.color_management.output_objects, &monitor);
        }
    }

    pub fn preferred_record(&self, surface: &WlSurface) -> Arc<Record> {
        self.surface_output(surface).map_or_else(
            || self.color_management.sdr(),
            |monitor| self.color_management.record_for(&monitor),
        )
    }

    fn refresh_preferred(&mut self) {
        let feedbacks = &mut self.color_management.feedbacks;
        feedbacks.retain(|feedback| feedback.upgrade().is_ok());
        let alive: Vec<_> = feedbacks
            .iter()
            .filter_map(|feedback| feedback.upgrade().ok())
            .collect();
        for object in alive {
            let Some(data) = object.data::<FeedbackData>() else {
                continue;
            };
            let Ok(surface) = data.surface.upgrade() else {
                continue;
            };
            let identity = self.preferred_record(&surface).identity;
            if data.replace(identity) != identity {
                feedback::notify(&object, identity);
            }
        }
    }
}
