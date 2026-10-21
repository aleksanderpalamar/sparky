use std::path::PathBuf;

use sparky_theme::{locations, wallpapers};

use crate::state::Sparky;
use crate::wallpaper::Decoded;
use crate::wallpaper::Settled;
use crate::wallpaper::choice::{choice, physical};

impl Sparky {
    pub fn refresh_wallpaper(&mut self) {
        let home = std::env::var("HOME").unwrap_or_default();
        let chosen = choice(self.config.general.wallpaper.as_deref(), &home, || self.wallpaper.fallback(default_wallpaper));
        let outputs = self.wallpaper_targets();
        for wanted in self.wallpaper.plan(chosen, &outputs) {
            self.wallpaper.decode(wanted);
        }
        self.needs_render = true;
    }

    pub fn receive_wallpaper(&mut self, decoded: Decoded) {
        match self.wallpaper.settle(decoded) {
            Settled::Painted => self.needs_render = true,
            Settled::Failed(error) => tracing::warn!("papel de parede: {error}"),
            Settled::Stale => {}
        }
    }

    fn wallpaper_targets(&self) -> Vec<(String, (u32, u32))> {
        self.space
            .outputs()
            .filter_map(|output| {
                let geometry = self.space.output_geometry(output)?;
                Some((output.name(), physical(geometry.size, output.current_scale().fractional_scale())))
            })
            .collect()
    }
}

fn default_wallpaper() -> Option<PathBuf> {
    wallpapers::of_theme(&wallpapers::roots_from_env(), locations::DEFAULT_THEME).into_iter().next()
}
