use std::fmt;

use sparky_theme::{Mode, Palette, Rgb, colors_file};

use crate::domain::system::{Change, Control, Row, RowId};
use crate::ports::themes::ThemeStore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub name: String,
    contents: String,
    palette: Palette,
    icons: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidTheme {
    Empty,
    Mode,
    Color(&'static str),
}

impl fmt::Display for InvalidTheme {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(formatter, "nenhuma cor reconhecida"),
            Self::Mode => write!(formatter, "modo de paleta inválido"),
            Self::Color(key) => write!(formatter, "cor inválida: {key}"),
        }
    }
}

impl std::error::Error for InvalidTheme {}

impl Theme {
    pub fn builtin() -> Self {
        Self {
            name: "tokyo-night".to_owned(),
            contents: include_str!("../../../../../data/themes/tokyo-night/colors.toml").to_owned(),
            palette: Palette::TOKYO_NIGHT,
            icons: None,
        }
    }

    pub fn parse(name: &str, contents: &str) -> Result<Self, InvalidTheme> {
        if colors_file::value(contents, "mode").is_some_and(|mode| !matches!(mode, "light" | "dark")) {
            return Err(InvalidTheme::Mode);
        }
        let keys = ["background", "lighter_background", "foreground", "bright_foreground", "dark_foreground", "accent", "selection", "muted", "red", "cyan"];
        let mut found = false;
        for key in keys {
            let Some(value) = colors_file::value(contents, key) else {
                continue;
            };
            if Rgb::parse(value).is_none() {
                return Err(InvalidTheme::Color(key));
            }
            found = true;
        }
        if !found {
            return Err(InvalidTheme::Empty);
        }
        Ok(Self { name: name.to_owned(), contents: contents.to_owned(), palette: Palette::from_colors(contents), icons: None })
    }

    pub fn palette(&self) -> Palette {
        self.palette
    }

    pub fn contents(&self) -> &str {
        &self.contents
    }

    pub fn with_icons(mut self, icons: Option<String>) -> Self {
        self.icons = icons;
        self
    }

    pub fn icons(&self) -> Option<&str> {
        self.icons.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub themes: Vec<Theme>,
    pub current: Palette,
}

impl Default for Catalog {
    fn default() -> Self {
        Self { themes: vec![Theme::builtin()], current: Palette::TOKYO_NIGHT }
    }
}

#[derive(Debug, Clone)]
pub enum Themes {
    Ready(Catalog),
    Unavailable(String),
}

impl Default for Themes {
    fn default() -> Self {
        Self::Ready(Catalog::default())
    }
}

impl Themes {
    pub fn observe(&mut self, palette: Palette) {
        if let Self::Ready(catalog) = self {
            catalog.current = palette;
        }
    }

    pub fn load(store: &dyn ThemeStore) -> Self {
        match store.load() {
            Ok(catalog) => Self::Ready(catalog),
            Err(error) => Self::Unavailable(error.to_string()),
        }
    }

    pub fn rows(&self) -> Vec<Row> {
        let catalog = match self {
            Self::Ready(catalog) => catalog,
            Self::Unavailable(error) => return vec![Row { id: RowId::ThemeStatus, label: "Paletas".to_owned(), value: error.clone(), control: Control::Reading }],
        };
        catalog.themes.iter().enumerate().map(|(index, theme)| {
            let active = theme.palette() == catalog.current;
            let mode = match theme.palette().mode { Mode::Dark => "escuro", Mode::Light => "claro" };
            let action = match active { true => "em uso", false => "aplicar" };
            Row { id: RowId::Theme(index), label: theme.name.clone(), value: format!("{mode} · {action}"), control: Control::Toggle(active) }
        }).collect()
    }

    pub fn request(&self, id: RowId, change: Change) -> Option<Theme> {
        let (Self::Ready(catalog), RowId::Theme(index), Change::Activate) = (self, id, change) else {
            return None;
        };
        catalog.themes.get(index).filter(|theme| theme.palette() != catalog.current).cloned()
    }
}
