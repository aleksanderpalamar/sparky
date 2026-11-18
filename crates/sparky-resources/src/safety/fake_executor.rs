use std::cell::RefCell;
use std::collections::HashMap;

use crate::domain::{Change, ChangeKind, Identity, IoClass};
use crate::ports::{ExecError, Executor};

#[derive(Default)]
pub struct FakeExecutor {
    pub values: RefCell<HashMap<(u32, ChangeKind), Change>>,
    pub tasks: RefCell<Vec<u32>>,
    pub deny: RefCell<bool>,
    pub ignore_writes: RefCell<bool>,
}

impl FakeExecutor {
    pub fn with_tasks(tasks: &[u32]) -> Self {
        let fake = Self::default();
        for task in tasks {
            fake.values
                .borrow_mut()
                .insert((*task, ChangeKind::Nice), Change::Nice(0));
            fake.values.borrow_mut().insert(
                (*task, ChangeKind::IoPriority),
                Change::IoPriority(IoClass::None),
            );
        }
        *fake.tasks.borrow_mut() = tasks.to_vec();
        fake
    }
}

impl Executor for FakeExecutor {
    fn tasks(&self, _target: Identity) -> Result<Vec<u32>, ExecError> {
        let tasks = self.tasks.borrow().clone();
        if tasks.is_empty() {
            return Err(ExecError::Gone);
        }
        Ok(tasks)
    }

    fn read(&self, task: u32, kind: ChangeKind) -> Result<Change, ExecError> {
        self.values
            .borrow()
            .get(&(task, kind))
            .cloned()
            .ok_or(ExecError::Gone)
    }

    fn write(&self, task: u32, change: &Change) -> Result<(), ExecError> {
        if *self.deny.borrow() {
            return Err(ExecError::Denied);
        }
        if !*self.ignore_writes.borrow() {
            self.values
                .borrow_mut()
                .insert((task, change.kind()), change.clone());
        }
        Ok(())
    }
}
