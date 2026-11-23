use std::cell::RefCell;
use std::rc::Rc;

use sparky_theme::Skin;

use super::state::Dock;
use crate::domain::app::{App, Launch};
use crate::ports::{GlassBoard, PinnedStore, Starter};

#[derive(Clone, Default)]
pub struct Memory {
    pub stored: Rc<RefCell<Option<Vec<String>>>>,
    pub saves: Rc<RefCell<usize>>,
    pub broken: bool,
}

impl PinnedStore for Memory {
    fn load(&self) -> Result<Option<Vec<String>>, String> {
        match self.broken {
            true => Err("ilegível".to_owned()),
            false => Ok(self.stored.borrow().clone()),
        }
    }

    fn save(&self, ids: &[String]) -> Result<(), String> {
        *self.stored.borrow_mut() = Some(ids.to_vec());
        *self.saves.borrow_mut() += 1;
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct Started(pub Rc<RefCell<Vec<Launch>>>);

impl Starter for Started {
    fn start(&self, launch: &Launch) -> Result<(), String> {
        self.0.borrow_mut().push(launch.clone());
        Ok(())
    }
}

pub fn dock(memory: &Memory, started: &Started, apps: Vec<App>) -> Dock {
    let mut dock = Dock::new(
        Skin::default(),
        Box::new(memory.clone()),
        Box::new(started.clone()),
        Box::new(Board::default()),
    );
    dock.apps = apps;
    dock
}

#[derive(Clone, Default)]
pub struct Board(pub Rc<RefCell<Vec<sparky_ipc::GlassArea>>>);

impl GlassBoard for Board {
    fn announce(&self, area: sparky_ipc::GlassArea) {
        self.0.borrow_mut().push(area);
    }
}
