use sparky_theme::{Palette, Skin};

use crate::app::state::Settings;
use crate::domain::system::themes::{Theme, Themes};
use crate::domain::system::{Change, Request, perform};

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
        self.report_theme(&theme);
    }

    fn report_theme(&mut self, theme: &Theme) {
        let Some(icons) = theme.icons() else {
            return self.inform(format!("Paleta {} aplicada.", theme.name));
        };
        match perform(self.services.commands.as_ref(), &Request::SetIconTheme(icons.to_owned())) {
            Ok(()) => self.inform(format!("Paleta {} aplicada com os ícones {icons}.", theme.name)),
            Err(error) => self.fail(format!("Paleta {} aplicada, mas os ícones {icons} não: {error}", theme.name)),
        }
    }
}
