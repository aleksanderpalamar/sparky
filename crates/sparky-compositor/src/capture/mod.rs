pub mod area;
mod pixels;
mod png_file;
mod render;

#[cfg(test)]
mod area_tests;

use std::path::Path;

use smithay::output::Output;
use sparky_ipc::CaptureArea;
use sparky_tiling::Rect;

use crate::capture::area::{CaptureError, resolve};
use crate::layout::outputs::rect_of;
use crate::state::Sparky;

impl Sparky {
    pub fn capture(&mut self, area: &CaptureArea, path: &Path) -> Result<(), CaptureError> {
        let screens = self.screens();
        let focused = self.desktop.focused_monitor().map(|monitor| monitor.name.clone());
        let target = resolve(area, &screens, focused.as_deref(), self.focused_window_rect())?;
        let output = self.output_named(&target.output).ok_or(CaptureError::NoOutput)?;
        let image = self
            .with_renderer(|state, renderer| render::read(state, renderer, &output, target.region))
            .ok_or_else(|| CaptureError::Render("o vídeo ainda não está pronto".to_owned()))??;
        png_file::write(path, &image)
    }

    fn screens(&self) -> Vec<(String, Rect)> {
        self.space
            .outputs()
            .filter_map(|output| self.space.output_geometry(output).map(|geometry| (output.name(), rect_of(geometry))))
            .collect()
    }

    fn output_named(&self, name: &str) -> Option<Output> {
        self.space.outputs().find(|output| output.name() == name).cloned()
    }

    fn focused_window_rect(&self) -> Option<Rect> {
        let window = self.windows.get(self.desktop.focused_window()?)?;
        self.space.element_geometry(window).map(rect_of)
    }
}
