use super::conductor::Conductor;
use super::log::Decision;
use crate::ports::{Executor, Storage};

impl<E: Executor, S: Storage> Conductor<E, S> {
    pub fn decisions(&self) -> Vec<Decision> {
        self.log.entries()
    }

    pub fn recorded(&self) -> u64 {
        self.log.recorded()
    }

    pub fn adjusting(&self) -> bool {
        self.active.is_some()
    }

    pub fn unconfirmed(&self) -> bool {
        self.unconfirmed
    }
}
