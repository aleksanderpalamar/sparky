use std::io;

use crate::domain::system::themes::{Catalog, Theme};

pub trait ThemeStore {
    fn load(&self) -> io::Result<Catalog>;
    fn apply(&self, theme: &Theme) -> io::Result<()>;
}
