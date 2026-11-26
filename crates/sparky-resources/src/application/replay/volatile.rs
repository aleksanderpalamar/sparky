use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::io;

use crate::domain::{Change, ChangeKind, Identity, IoClass};
use crate::ports::{ExecError, Executor, Storage};

const CPUS: [u32; 4] = [0, 1, 2, 3];

#[derive(Default)]
pub struct Recorder {
    values: RefCell<HashMap<(u32, ChangeKind), Change>>,
    writes: RefCell<Vec<u32>>,
}

impl Recorder {
    pub fn writes_to(&self, task: u32) -> usize {
        self.writes
            .borrow()
            .iter()
            .filter(|written| **written == task)
            .count()
    }
}

impl Executor for &Recorder {
    fn tasks(&self, target: Identity) -> Result<Vec<u32>, ExecError> {
        Ok(vec![target.pid])
    }

    fn read(&self, task: u32, kind: ChangeKind) -> Result<Change, ExecError> {
        Ok(self
            .values
            .borrow()
            .get(&(task, kind))
            .cloned()
            .unwrap_or_else(|| initial(kind)))
    }

    fn write(&self, task: u32, change: &Change) -> Result<(), ExecError> {
        self.writes.borrow_mut().push(task);
        self.values
            .borrow_mut()
            .insert((task, change.kind()), change.clone());
        Ok(())
    }
}

#[derive(Default)]
pub struct Volatile {
    files: RefCell<BTreeMap<String, Vec<u8>>>,
}

impl Storage for &Volatile {
    fn load(&self, name: &str) -> io::Result<Vec<u8>> {
        self.files
            .borrow()
            .get(name)
            .cloned()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }

    fn save(&self, name: &str, bytes: &[u8]) -> io::Result<()> {
        self.files
            .borrow_mut()
            .insert(name.to_owned(), bytes.to_vec());
        Ok(())
    }

    fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        let bytes = self.load(from)?;
        let mut files = self.files.borrow_mut();
        files.remove(from);
        files.insert(to.to_owned(), bytes);
        Ok(())
    }

    fn remove(&self, name: &str) -> io::Result<()> {
        self.files.borrow_mut().remove(name);
        Ok(())
    }
}

fn initial(kind: ChangeKind) -> Change {
    match kind {
        ChangeKind::Nice => Change::Nice(0),
        ChangeKind::IoPriority => Change::IoPriority(IoClass::None),
        ChangeKind::Affinity => Change::Affinity(CPUS.to_vec()),
    }
}
