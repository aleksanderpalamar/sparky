use std::path::Path;

use sparky_theme::{Palette, Skin};
use sparky_ui::notice::Notice;

use crate::app::state::Settings;
use crate::domain::field::Field;
use crate::domain::system::themes::{Theme, Themes};
use crate::domain::system::wallpaper;
use crate::domain::system::{Change, Request, Row, RowId, perform};
use crate::domain::value::{Step, Value};

impl Settings {
    pub fn observe_palette(&mut self, palette: Palette) {
        self.context.skin = Skin::new(palette);
        self.themes.observe(palette);
    }

    pub fn theme_rows(&self) -> Vec<Row> {
        let configured = self.config.general.wallpaper.as_deref();
        wallpaper::row(&self.themes, configured).into_iter().chain(self.themes.rows()).collect()
    }

    pub fn change_theme(&mut self, change: Change) {
        let Some(row) = self.system_rows().into_iter().nth(self.focus) else { return };
        if row.id == RowId::Wallpaper {
            return self.step_wallpaper(change);
        }
        let Some(theme) = self.themes.request(row.id, change) else { return };
        if let Err(error) = self.services.themes.apply(&theme) {
            self.fail(format!("Paleta {}: {error}", theme.name));
            return;
        }
        self.themes = Themes::load(self.services.themes.as_ref());
        self.observe_palette(theme.palette());
        self.notice = None;
        if let Some(first) = theme.wallpapers().first() {
            self.choose_wallpaper(first);
        }
        self.focus_row(row.id);
        if !matches!(self.notice, Some(Notice::Error(_))) {
            self.report_theme(&theme);
        }
    }

    fn step_wallpaper(&mut self, change: Change) {
        let step = match change {
            Change::Step(step) => step,
            Change::Activate | Change::Text(_) => Step::More,
        };
        if let Some(path) = wallpaper::step(&self.themes, self.config.general.wallpaper.as_deref(), step) {
            self.choose_wallpaper(&path);
        }
    }

    fn choose_wallpaper(&mut self, path: &Path) {
        self.set(Field::Wallpaper, Value::Text(path.to_string_lossy().into_owned()));
    }

    fn focus_row(&mut self, id: RowId) {
        let index = self.system_rows().iter().position(|row| row.id == id);
        self.focus = index.unwrap_or(self.focus);
        self.move_focus(0);
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
