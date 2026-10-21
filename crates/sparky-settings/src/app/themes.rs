use sparky_theme::{Palette, Skin};

use crate::app::state::Settings;
use crate::domain::system::Change;
use crate::domain::system::themes::Themes;

impl Settings {
    pub fn observe_palette(&mut self, palette: Palette) {
        self.context.skin = Skin::new(palette);
        self.themes.observe(palette);
    }

    pub fn change_theme(&mut self, change: Change) {
        let Some(row) = self.system_rows().into_iter().nth(self.focus) else { return };
        let Some(theme) = self.themes.request(row.id, change) else { return };
        if let Err(error) = self.services.themes.apply(&theme) {
            self.fail(format!("Paleta {}: {error}", theme.name));
            return;
        }
        self.themes = Themes::load(self.services.themes.as_ref());
        self.observe_palette(theme.palette());
        self.move_focus(0);
        self.inform(format!("Paleta {} aplicada.", theme.name));
    }
}
