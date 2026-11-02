use smithay::output::Output;

use crate::backend::Backend;
use crate::color_management::presets::OutputColor;
use crate::state::Sparky;

impl Sparky {
    pub fn output_colors(&self) -> Option<Vec<(Output, OutputColor)>> {
        match &self.backend {
            Backend::Drm(drm) => Some(drm.output_colors()),
            Backend::Nested(nested) => Some(vec![(nested.output.clone(), OutputColor::Sdr)]),
            Backend::Starting => None,
        }
    }
}
