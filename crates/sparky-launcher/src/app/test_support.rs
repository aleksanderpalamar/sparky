use std::cell::RefCell;
use std::rc::Rc;

use sparky_theme::Skin;

use crate::app::state::Launcher;
use crate::domain::app::{App, Runs};
use crate::domain::groups::Group;
use crate::domain::mode::Mode;
use crate::ports::GroupStore;

#[derive(Clone, Default)]
pub struct Memory(pub Rc<RefCell<Vec<Group>>>);

impl GroupStore for Memory {
    fn load(&self) -> Result<Vec<Group>, String> {
        Ok(self.0.borrow().clone())
    }

    fn save(&self, groups: &[Group]) -> Result<(), String> {
        *self.0.borrow_mut() = groups.to_vec();
        Ok(())
    }
}

pub fn app(name: &str, categories: &[&str]) -> App {
    App {
        id: name.to_lowercase(),
        name: name.to_owned(),
        detail: None,
        keywords: Vec::new(),
        command: name.to_lowercase(),
        runs: Runs::Directly,
        icon: None,
        categories: categories.iter().map(|name| (*name).to_owned()).collect(),
    }
}

pub fn launcher(names: &[&str], memory: &Memory) -> Launcher {
    let apps = names.iter().map(|name| app(name, &[])).collect();
    Launcher::new(apps, Skin::default(), Mode::Apps, Box::new(memory.clone()))
}
