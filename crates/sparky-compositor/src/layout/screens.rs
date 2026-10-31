use smithay::output::Mode;
use sparky_config::OutputMode;
use sparky_ipc::{Hdr, Vrr};

use crate::backend::Backend;
use crate::state::Sparky;

pub struct Screen {
    pub name: String,
    pub description: String,
    pub modes: Vec<OutputMode>,
    pub vrr: Vrr,
    pub enabled: bool,
    pub hdr: Hdr,
}

pub fn output_mode(mode: Mode) -> OutputMode {
    OutputMode {
        width: mode.size.w.max(0) as u32,
        height: mode.size.h.max(0) as u32,
        refresh: mode.refresh.max(0) as u32,
    }
}

impl Sparky {
    pub fn connected_screens(&self) -> Vec<Screen> {
        if let Backend::Drm(drm) = &self.backend {
            return drm.screens();
        }
        self.space
            .outputs()
            .map(|output| Screen {
                name: output.name(),
                description: String::new(),
                modes: output.current_mode().map(output_mode).into_iter().collect(),
                vrr: Vrr::Unsupported,
                enabled: true,
                hdr: Hdr::Unsupported,
            })
            .collect()
    }
}
