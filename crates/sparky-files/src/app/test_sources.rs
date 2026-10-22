use sparky_storage::{Operation, Volume, VolumeSource};

use crate::domain::icons::Icons;
use crate::ports::IconSource;

pub struct NoIcons;

impl IconSource for NoIcons {
    fn load(&mut self) -> Icons {
        Icons::default()
    }
}

pub struct NoVolumes;

impl VolumeSource for NoVolumes {
    fn run(&self, _operation: Operation) -> Result<Vec<Volume>, String> {
        Ok(Vec::new())
    }
}
