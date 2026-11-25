use std::io;

use super::checkpoint::{Checkpoint, CheckpointError, Kind, decode, encode};
use crate::ports::Storage;

#[derive(Debug, Clone, PartialEq)]
pub enum Restored {
    Model(Checkpoint),
    Missing,
    Unreadable,
    Rejected(CheckpointError),
    OtherHardware,
}

pub struct Knowledge<S> {
    storage: S,
    written: u64,
}

impl<S: Storage> Knowledge<S> {
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            written: 0,
        }
    }

    pub fn written(&self) -> u64 {
        self.written
    }

    pub fn restore(&self, kind: Kind, hardware: u64) -> Restored {
        self.restore_with(kind, hardware, |_| true)
    }

    pub fn restore_with(
        &self,
        kind: Kind,
        hardware: u64,
        accept: impl Fn(&Checkpoint) -> bool,
    ) -> Restored {
        let current = self.read(&active(kind), kind, hardware, &accept);
        if matches!(current, Restored::Model(_) | Restored::OtherHardware) {
            return current;
        }
        match self.read(&previous(kind), kind, hardware, &accept) {
            Restored::Model(model) => Restored::Model(model),
            fallback if current == Restored::Missing => fallback,
            _ => current,
        }
    }

    pub fn store(&mut self, checkpoint: &Checkpoint) -> io::Result<()> {
        let kind = checkpoint.kind;
        let bytes = encode(checkpoint);
        self.storage.save(&next(kind), &bytes)?;
        match self.storage.rename(&active(kind), &previous(kind)) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
        self.storage.rename(&next(kind), &active(kind))?;
        self.written = self.written.saturating_add(bytes.len() as u64);
        Ok(())
    }

    pub fn roll_back(&mut self, kind: Kind) -> io::Result<()> {
        self.storage.rename(&previous(kind), &active(kind))
    }

    pub fn forget(&mut self) -> io::Result<()> {
        let results: Vec<io::Result<()>> = Kind::ALL
            .into_iter()
            .map(|kind| self.discard(kind))
            .collect();
        results.into_iter().collect()
    }

    pub fn discard(&mut self, kind: Kind) -> io::Result<()> {
        let results: Vec<io::Result<()>> = [active(kind), previous(kind), next(kind)]
            .iter()
            .map(|name| match self.storage.remove(name) {
                Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
                _ => Ok(()),
            })
            .collect();
        results.into_iter().collect()
    }

    fn read(
        &self,
        name: &str,
        kind: Kind,
        hardware: u64,
        accept: &impl Fn(&Checkpoint) -> bool,
    ) -> Restored {
        let bytes = match self.storage.load(name) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Restored::Missing,
            Err(_) => return Restored::Unreadable,
        };
        match decode(&bytes) {
            Ok(model) if model.kind != kind => Restored::Rejected(CheckpointError::Kind),
            Ok(model) if model.hardware != hardware => Restored::OtherHardware,
            Ok(model) if !accept(&model) => Restored::Rejected(CheckpointError::Values),
            Ok(model) => Restored::Model(model),
            Err(error) => Restored::Rejected(error),
        }
    }
}

fn active(kind: Kind) -> String {
    format!("{}.ckpt", name(kind))
}

fn next(kind: Kind) -> String {
    format!("{}.next.ckpt", name(kind))
}

fn previous(kind: Kind) -> String {
    format!("{}.previous.ckpt", name(kind))
}

fn name(kind: Kind) -> &'static str {
    match kind {
        Kind::Sequence => "sequence",
        Kind::Context => "context",
        Kind::Candidate => "candidate",
        Kind::History => "history",
        Kind::Rounds => "rounds",
    }
}
