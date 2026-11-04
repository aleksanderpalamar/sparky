use std::path::Path;

use sparky_theme::{Palette, Skin};
use sparky_ui::notice::Notice;

use crate::app::state::Settings;
use crate::domain::field::Field;
use crate::domain::system::themes::{Theme, Themes};
use crate::domain::system::wallpaper;
use crate::domain::system::{Change, Request, Row, RowId, perform};
use crate::domain::value::{Step, Value};

const WALLPAPERS: [RowId; 2] = [RowId::Wallpaper, RowId::LoginWallpaper];

impl Settings {
    pub fn observe_palette(&mut self, palette: Palette) {
        self.context.skin = Skin::new(palette);
        self.themes.observe(palette);
    }

    pub fn theme_rows(&self) -> Vec<Row> {
        let wallpapers = WALLPAPERS.into_iter().filter_map(|id| {
            let (_, configured) = self.wallpaper_of(id)?;
            wallpaper::row(&self.themes, configured, id)
        });
        self.themes.rows().into_iter().chain(wallpapers).collect()
    }

    pub fn change_theme(&mut self, change: Change) {
        let Some(row) = self.system_rows().into_iter().nth(self.focus) else {
            return;
        };
        if WALLPAPERS.contains(&row.id) {
            return self.change_wallpaper(row.id, change);
        }
        let Some(theme) = self.themes.request(row.id, change) else {
            return;
        };
        if let Err(error) = self.services.themes.apply(&theme) {
            self.fail(theme_message(
                "theme-failed",
                &theme.name,
                None,
                Some(&error.to_string()),
            ));
            return;
        }
        self.themes = Themes::load(self.services.themes.as_ref());
        self.observe_palette(theme.palette());
        self.notice = None;
        if let Some(first) = theme.wallpapers().first() {
            self.choose_wallpaper(Field::Wallpaper, first);
            self.choose_wallpaper(Field::LoginWallpaper, first);
        }
        self.focus_row(row.id);
        if !matches!(self.notice, Some(Notice::Error(_))) {
            self.report_theme(&theme);
        }
    }

    fn change_wallpaper(&mut self, id: RowId, change: Change) {
        let Some((field, configured)) = self.wallpaper_of(id) else {
            return;
        };
        let path = match change {
            Change::Text(name) => wallpaper::chosen(&self.themes, &name),
            Change::Step(step) => wallpaper::step(&self.themes, configured, step),
            Change::Activate => wallpaper::step(&self.themes, configured, Step::More),
        };
        if let Some(path) = path {
            self.choose_wallpaper(field, &path);
        }
    }

    fn wallpaper_of(&self, id: RowId) -> Option<(Field, Option<&str>)> {
        match id {
            RowId::Wallpaper => Some((Field::Wallpaper, self.config.general.wallpaper.as_deref())),
            RowId::LoginWallpaper => Some((Field::LoginWallpaper, self.config.login_wallpaper())),
            _ => None,
        }
    }

    fn choose_wallpaper(&mut self, field: Field, path: &Path) {
        self.set(field, Value::Text(path.to_string_lossy().into_owned()));
    }

    fn focus_row(&mut self, id: RowId) {
        let index = self.system_rows().iter().position(|row| row.id == id);
        self.focus = index.unwrap_or(self.focus);
        self.move_focus(0);
    }

    fn report_theme(&mut self, theme: &Theme) {
        let Some(icons) = theme.icons() else {
            return self.inform(theme_message("theme-applied", &theme.name, None, None));
        };
        match perform(
            self.services.commands.as_ref(),
            &Request::SetIconTheme(icons.to_owned()),
        ) {
            Ok(()) => self.inform(theme_message(
                "theme-icons-applied",
                &theme.name,
                Some(icons),
                None,
            )),
            Err(error) => self.fail(theme_message(
                "theme-icons-failed",
                &theme.name,
                Some(icons),
                Some(&error.to_string()),
            )),
        }
    }
}

fn theme_message(key: &str, name: &str, icons: Option<&str>, reason: Option<&str>) -> String {
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("name", name);
    if let Some(icons) = icons {
        args.set("icons", icons);
    }
    if let Some(reason) = reason {
        args.set("reason", reason);
    }
    crate::i18n::message(key, Some(&args))
}
