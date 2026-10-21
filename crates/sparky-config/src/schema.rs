use serde::{Deserialize, Serialize};
use sparky_tiling::{Gaps, Style};

use crate::action::Bind;
use crate::output::OutputRule;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub general: General,
    pub input: Input,
    pub apps: Apps,
    pub autostart: Vec<String>,
    pub bind: Vec<Bind>,
    pub output: Vec<OutputRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct General {
    pub gaps_inner: i32,
    pub gaps_outer: i32,
    pub border: i32,
    pub float_width: i32,
    pub float_height: i32,
    pub default_binds: bool,
    pub wallpaper: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Input {
    pub layout: Option<String>,
    pub variant: Option<String>,
    pub model: Option<String>,
    pub options: Option<String>,
    pub repeat_rate: i32,
    pub repeat_delay: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Apps {
    pub terminal: Option<String>,
    pub launcher: String,
    pub files: String,
    pub settings: String,
    pub screenshot: String,
    pub locker: String,
}

impl General {
    pub const SPARKY: Self = Self {
        gaps_inner: Style::SPARKY.gaps.inner,
        gaps_outer: Style::SPARKY.gaps.outer,
        border: 2,
        float_width: Style::SPARKY.float_size.0,
        float_height: Style::SPARKY.float_size.1,
        default_binds: true,
        wallpaper: None,
    };

    pub fn style(&self) -> Style {
        Style {
            gaps: Gaps {
                inner: self.gaps_inner,
                outer: self.gaps_outer,
            },
            float_size: (self.float_width, self.float_height),
        }
    }
}

impl Input {
    pub const SPARKY: Self = Self {
        layout: None,
        variant: None,
        model: None,
        options: None,
        repeat_rate: 40,
        repeat_delay: 250,
    };
}

impl Default for General {
    fn default() -> Self {
        Self::SPARKY
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::SPARKY
    }
}

impl Default for Apps {
    fn default() -> Self {
        Self {
            terminal: None,
            launcher: "sparky-launcher".to_owned(),
            files: "sparky-files".to_owned(),
            settings: "sparky-settings".to_owned(),
            screenshot: "sparky-shot".to_owned(),
            locker: "swaylock".to_owned(),
        }
    }
}
