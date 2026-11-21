use super::journal::{Applied, Outcome, restore};
use crate::domain::Identity;
use crate::ports::{Executor, Storage};

const NAME: &str = "applied.json";

pub struct Ledger<S> {
    storage: S,
    entries: Vec<Applied>,
}

impl<S: Storage> Ledger<S> {
    pub fn open(storage: S) -> Self {
        let entries = storage
            .load(NAME)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self { storage, entries }
    }

    pub fn entries(&self) -> &[Applied] {
        &self.entries
    }

    pub fn record(&mut self, applied: Applied) -> std::io::Result<()> {
        self.entries.push(applied);
        self.save()
    }

    pub fn settle(&mut self, target: Identity) -> std::io::Result<()> {
        self.entries
            .retain(|entry| entry.adjustment.target != target);
        self.save()
    }

    pub fn recover(&mut self, executor: &impl Executor) -> Vec<Outcome> {
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

    fn save(&self) -> std::io::Result<()> {
        let bytes = serde_json::to_vec(&self.entries).map_err(std::io::Error::other)?;
        self.storage.save(NAME, &bytes)
    }
}
