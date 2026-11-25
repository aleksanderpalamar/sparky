use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;

use crate::ports::Storage;

#[derive(Default)]
pub struct FakeStorage {
    pub files: RefCell<BTreeMap<String, Vec<u8>>>,
    pub refuse: RefCell<bool>,
    pub unreadable: RefCell<bool>,
    pub broken: RefCell<BTreeSet<String>>,
}

fn missing() -> io::Error {
    io::Error::from(io::ErrorKind::NotFound)
}

impl FakeStorage {
    pub fn break_file(&self, name: &str) {
        self.broken.borrow_mut().insert(name.to_owned());
    }

    pub fn mend(&self) {
        self.broken.borrow_mut().clear();
    }

    fn touch(&self, name: &str) -> io::Result<()> {
        match self.broken.borrow().contains(name) {
            true => Err(io::Error::other("falha simulada")),
            false => Ok(()),
        }
    }
}

impl Storage for &FakeStorage {
    fn load(&self, name: &str) -> io::Result<Vec<u8>> {
        self.touch(name)?;
        if *self.unreadable.borrow() {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
        self.files.borrow().get(name).cloned().ok_or_else(missing)
    }

    fn save(&self, name: &str, bytes: &[u8]) -> io::Result<()> {
        self.touch(name)?;
        if *self.refuse.borrow() {
            return Err(io::Error::other("armazenamento recusado"));
        }
        self.files
            .borrow_mut()
            .insert(name.to_owned(), bytes.to_vec());
        Ok(())
    }

    fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        self.touch(from)?;
        self.touch(to)?;
        let mut files = self.files.borrow_mut();
        let bytes = files.remove(from).ok_or_else(missing)?;
        files.insert(to.to_owned(), bytes);
        Ok(())
    }

    fn remove(&self, name: &str) -> io::Result<()> {
        self.touch(name)?;
        self.files
            .borrow_mut()
            .remove(name)
            .map(drop)
            .ok_or_else(missing)
    }
}
