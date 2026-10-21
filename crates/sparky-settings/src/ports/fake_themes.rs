use std::cell::{Cell, RefCell};
use std::io;
use std::rc::Rc;

use crate::domain::system::themes::{Catalog, Theme};
use crate::ports::themes::ThemeStore;

#[derive(Default)]
pub struct FakeThemes {
    pub catalog: RefCell<Catalog>,
    pub applied: RefCell<Vec<String>>,
    pub failure: Cell<Option<io::ErrorKind>>,
}

impl ThemeStore for Rc<FakeThemes> {
    fn load(&self) -> io::Result<Catalog> {
        Ok(self.catalog.borrow().clone())
    }

    fn apply(&self, theme: &Theme) -> io::Result<()> {
        if let Some(kind) = self.failure.get() {
            return Err(io::Error::from(kind));
        }
        self.applied.borrow_mut().push(theme.name.clone());
        self.catalog.borrow_mut().current = theme.palette();
        Ok(())
    }
}
