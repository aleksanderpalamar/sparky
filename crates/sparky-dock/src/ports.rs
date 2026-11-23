use sparky_ipc::GlassArea;

use crate::domain::app::Launch;

pub trait PinnedStore {
    fn load(&self) -> Result<Option<Vec<String>>, String>;
    fn save(&self, ids: &[String]) -> Result<(), String>;
}

pub trait Starter {
    fn start(&self, launch: &Launch) -> Result<(), String>;
}

pub trait GlassBoard {
    fn announce(&self, area: GlassArea);
}
