use std::time::Duration;

use sparky_theme::Rgb;

use crate::domain::field::Field;
use crate::run::Limits;

const TIMEOUT: Duration = Duration::from_millis(400);
const WORKERS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Visual,
    Json,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Logo {
    Show,
    Hide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Classic,
    Stacked,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Colors {
    pub accent: Option<Rgb>,
    pub label: Option<Rgb>,
    pub value: Option<Rgb>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Partial {
    pub fields: Option<Vec<Field>>,
    pub hide: Vec<Field>,
    pub format: Option<Format>,
    pub color: Option<ColorMode>,
    pub logo: Option<Logo>,
    pub layout: Option<Layout>,
    pub width: Option<usize>,
    pub timeout: Option<Duration>,
    pub colors: Colors,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub fields: Vec<Field>,
    pub format: Option<Format>,
    pub color: ColorMode,
    pub logo: Logo,
    pub layout: Layout,
    pub width: Option<usize>,
    pub timeout: Duration,
    pub workers: usize,
    pub colors: Colors,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fields: Field::ALL.to_vec(),
            format: None,
            color: ColorMode::Auto,
            logo: Logo::Show,
            layout: Layout::Classic,
            width: None,
            timeout: TIMEOUT,
            workers: WORKERS,
            colors: Colors::default(),
        }
    }
}

impl Settings {
    pub fn merged(&self, partials: &[Partial]) -> Settings {
        partials.iter().fold(self.clone(), Settings::apply)
    }

    pub fn limits(&self) -> Limits {
        Limits {
            workers: self.workers,
            timeout: self.timeout,
        }
    }

    fn apply(mut self, partial: &Partial) -> Settings {
        if let Some(fields) = &partial.fields {
            self.fields = fields.clone();
        }
        self.fields.retain(|field| !partial.hide.contains(field));
        self.format = partial.format.or(self.format);
        self.color = partial.color.unwrap_or(self.color);
        self.logo = partial.logo.unwrap_or(self.logo);
        self.layout = partial.layout.unwrap_or(self.layout);
        self.width = partial.width.or(self.width);
        self.timeout = partial.timeout.unwrap_or(self.timeout);
        self.colors = Colors {
            accent: partial.colors.accent.or(self.colors.accent),
            label: partial.colors.label.or(self.colors.label),
            value: partial.colors.value.or(self.colors.value),
        };
        self
    }
}
