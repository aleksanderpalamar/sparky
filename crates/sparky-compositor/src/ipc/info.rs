use sparky_ipc::{SessionInfo, SessionKind};
use sparky_theme::locations::theme_roots_from_env;
use sparky_theme::names::{installed_themes, theme_name};
use sparky_theme::{Palette, Rgb};

use crate::state::Sparky;

impl Sparky {
    pub fn session_info(&self) -> SessionInfo {
        let palette = sparky_theme::current_palette();
        SessionInfo {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            display: self.socket_name.to_string_lossy().into_owned(),
            session: match self.is_real_session() {
                true => SessionKind::Real,
                false => SessionKind::Nested,
            },
            theme: theme_name(&palette, &installed_themes(&theme_roots_from_env())),
            accent: palette.accent.hex(),
            swatches: swatches(&palette),
        }
    }
}

fn swatches(palette: &Palette) -> Vec<String> {
    [
        palette.background,
        palette.urgent,
        palette.cyan,
        palette.accent,
        palette.muted,
        palette.foreground,
        palette.bright_foreground,
    ]
    .into_iter()
    .map(Rgb::hex)
    .collect()
}
