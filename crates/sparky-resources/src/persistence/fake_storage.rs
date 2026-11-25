use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;

use crate::ports::Storage;

#[derive(Default)]
pub struct FakeStorage {
    pub files: RefCell<BTreeMap<String, Vec<u8>>>,
    pub refuse: RefCell<bool>,
}

fn missing() -> io::Error {
    io::Error::from(io::ErrorKind::NotFound)
}

impl Storage for &FakeStorage {
    fn load(&self, name: &str) -> io::Result<Vec<u8>> {
        self.files.borrow().get(name).cloned().ok_or_else(missing)
    }

    fn save(&self, name: &str, bytes: &[u8]) -> io::Result<()> {
        if *self.refuse.borrow() {
            return Err(io::Error::other("armazenamento recusado"));
        }
        self.files
            .borrow_mut()
            .insert(name.to_owned(), bytes.to_vec());
        Ok(())
    }

    fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        let mut files = self.files.borrow_mut();
        let bytes = files.remove(from).ok_or_else(missing)?;
        files.insert(to.to_owned(), bytes);
        Ok(())
    }

    fn remove(&self, name: &str) -> io::Result<()> {
        self.files
            .borrow_mut()
            .remove(name)
            .map(drop)
            .ok_or_else(missing)
    }
}
