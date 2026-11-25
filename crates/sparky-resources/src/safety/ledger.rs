use super::journal::{Applied, Outcome, restore};
use crate::ports::{Executor, Storage};

const NAME: &str = "applied.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reading {
    Readable,
    Unreadable,
}

pub struct Ledger<S> {
    storage: S,
    entries: Vec<Applied>,
    reading: Reading,
}

impl<S: Storage> Ledger<S> {
    pub fn open(storage: S) -> Self {
        let (entries, reading) = load(&storage);
        Self {
            storage,
            entries,
            reading,
        }
    }

    pub fn entries(&self) -> &[Applied] {
        &self.entries
    }

    pub fn pending(&self) -> bool {
        self.reading == Reading::Unreadable || !self.entries.is_empty()
    }

    pub fn record(&mut self, applied: Applied) -> std::io::Result<()> {
        self.writable()?;
        self.entries.push(applied);
        let saved = self.save();
        if saved.is_err() {
            self.entries.pop();
        }
        saved
    }

    pub fn settle(&mut self, applied: &Applied) -> std::io::Result<()> {
        self.writable()?;
        let Some(index) = self.entries.iter().rposition(|entry| entry == applied) else {
            return Ok(());
        };
        self.entries.remove(index);
        self.save()
    }

    pub fn recover(&mut self, executor: &impl Executor) -> Vec<Outcome> {
        if !self.reread() {
            return vec![Outcome::Unconfirmed];
        }
        let outcomes: Vec<Outcome> = self
            .entries
            .iter()
            .rev()
            .map(|entry| restore(executor, entry))
            .collect();
        let pending: Vec<Applied> = self
            .entries
            .iter()
            .rev()
            .zip(&outcomes)
            .filter(|(_, outcome)| **outcome == Outcome::Unconfirmed)
            .map(|(entry, _)| entry.clone())
            .collect();
        self.entries = pending;
        if let Err(error) = self.save() {
            eprintln!("sparky-resources: registro de ajustes não gravado: {error}");
        }
        outcomes
    }

    fn reread(&mut self) -> bool {
        if self.reading == Reading::Readable {
            return true;
        }
        let (entries, reading) = load(&self.storage);
        self.entries = entries;
        self.reading = reading;
        reading == Reading::Readable
    }

    fn writable(&self) -> std::io::Result<()> {
        match self.reading {
            Reading::Readable => Ok(()),
            Reading::Unreadable => Err(std::io::Error::other("registro de ajustes ilegível")),
        }
    }

    fn save(&self) -> std::io::Result<()> {
        self.writable()?;
        let bytes = serde_json::to_vec(&self.entries).map_err(std::io::Error::other)?;
        self.storage.save(NAME, &bytes)
    }
}

fn load(storage: &impl Storage) -> (Vec<Applied>, Reading) {
    match storage.load(NAME) {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(entries) => (entries, Reading::Readable),
            Err(_) => (Vec::new(), Reading::Unreadable),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            (Vec::new(), Reading::Readable)
        }
        Err(_) => (Vec::new(), Reading::Unreadable),
    }
}
