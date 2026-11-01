use sparky_theme::Palette;

use super::Theme;
use crate::domain::system::{Change, Control, Row, RowId};
use crate::domain::value::Step;
use crate::ports::themes::ThemeStore;

#[derive(Debug, Clone)]
pub struct Catalog {
    pub themes: Vec<Theme>,
    pub current: Palette,
}

impl Default for Catalog {
    fn default() -> Self {
        Self {
            themes: vec![Theme::builtin()],
            current: Palette::TOKYO_NIGHT,
        }
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
            Self::Unavailable(error) => {
                return vec![Row {
                    id: RowId::ThemeStatus,
                    label: "Paletas".to_owned(),
                    value: error.clone(),
                    control: Control::Reading,
                }];
            }
        };
        let value = catalog
            .active()
            .map_or_else(|| "outro".to_owned(), Theme::label);
        let options = catalog.themes.iter().map(Theme::label).collect();
        vec![Row {
            id: RowId::Theme,
            label: "Tema atual".to_owned(),
            value,
            control: Control::Choice(options),
        }]
    }

    pub fn request(&self, id: RowId, change: Change) -> Option<Theme> {
        let (Self::Ready(catalog), RowId::Theme) = (self, id) else {
            return None;
        };
        let chosen = match change {
            Change::Text(label) => catalog
                .themes
                .iter()
                .find(|theme| theme.label() == label.trim()),
            Change::Step(step) => catalog.neighbor(step),
            Change::Activate => None,
        };
        chosen
            .filter(|theme| theme.palette() != catalog.current)
            .cloned()
    }
}

impl Catalog {
    fn active(&self) -> Option<&Theme> {
        self.themes
            .iter()
            .find(|theme| theme.palette() == self.current)
    }

    fn neighbor(&self, step: Step) -> Option<&Theme> {
        let count = self.themes.len();
        let current = self
            .themes
            .iter()
            .position(|theme| theme.palette() == self.current)
            .unwrap_or(0);
        let next = match step {
            Step::Less => (current + count.checked_sub(1)?) % count,
            Step::More => (current + 1) % count,
        };
        self.themes.get(next)
    }
}
