use std::fmt;
use std::path::PathBuf;

use sparky_theme::{Mode, Palette, Rgb, colors_file};

mod catalog;

pub use catalog::{Catalog, Themes};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub name: String,
    contents: String,
    palette: Palette,
    icons: Option<String>,
    wallpapers: Vec<PathBuf>,
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
            wallpapers: Vec::new(),
        }
    }

    pub fn parse(name: &str, contents: &str) -> Result<Self, InvalidTheme> {
        if colors_file::value(contents, "mode")
            .is_some_and(|mode| !matches!(mode, "light" | "dark"))
        {
            return Err(InvalidTheme::Mode);
        }
        let keys = [
            "background",
            "lighter_background",
            "foreground",
            "bright_foreground",
            "dark_foreground",
            "accent",
            "selection",
            "muted",
            "red",
            "cyan",
        ];
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
        Ok(Self {
            name: name.to_owned(),
            contents: contents.to_owned(),
            palette: Palette::from_colors(contents),
            icons: None,
            wallpapers: Vec::new(),
        })
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

    pub fn with_wallpapers(mut self, wallpapers: Vec<PathBuf>) -> Self {
        self.wallpapers = wallpapers;
        self
    }

    pub fn label(&self) -> String {
        let mode = match self.palette.mode {
            Mode::Dark => "escuro",
            Mode::Light => "claro",
        };
        format!("{} ({mode})", self.name)
    }

    pub fn wallpapers(&self) -> &[PathBuf] {
        &self.wallpapers
    }
}
