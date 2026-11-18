use std::io;

use super::checkpoint::{Checkpoint, CheckpointError, Kind, decode, encode};
use crate::ports::Storage;

#[derive(Debug, Clone, PartialEq)]
pub enum Restored {
    Model(Checkpoint),
    Missing,
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
        match self.read(&active(kind), kind, hardware) {
            Restored::Rejected(_) => match self.read(&previous(kind), kind, hardware) {
                Restored::Model(model) => Restored::Model(model),
                _ => self.read(&active(kind), kind, hardware),
            },
            other => other,
        }
    }

    pub fn promote(&mut self, checkpoint: &Checkpoint) -> io::Result<()> {
        let bytes = encode(checkpoint);
        match self
            .storage
            .rename(&active(checkpoint.kind), &previous(checkpoint.kind))
        {
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
        self.storage.save(&active(checkpoint.kind), &bytes)?;
        self.written = self.written.saturating_add(bytes.len() as u64);
        Ok(())
    }

    pub fn roll_back(&mut self, kind: Kind) -> io::Result<()> {
        self.storage.rename(&previous(kind), &active(kind))
    }

    pub fn forget(&mut self) -> io::Result<()> {
        for kind in [Kind::Sequence, Kind::Context] {
            for name in [active(kind), previous(kind)] {
                match self.storage.remove(&name) {
                    Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn read(&self, name: &str, kind: Kind, hardware: u64) -> Restored {
        let bytes = match self.storage.load(name) {
            Ok(bytes) => bytes,
            Err(_) => return Restored::Missing,
        };
        match decode(&bytes) {
            Ok(model) if model.kind != kind => Restored::Rejected(CheckpointError::Kind),
            Ok(model) if model.hardware != hardware => Restored::OtherHardware,
            Ok(model) => Restored::Model(model),
            Err(error) => Restored::Rejected(error),
        }
    }
}

fn active(kind: Kind) -> String {
    format!("{}.ckpt", name(kind))
}

fn previous(kind: Kind) -> String {
    format!("{}.previous.ckpt", name(kind))
}

fn name(kind: Kind) -> &'static str {
    match kind {
        Kind::Sequence => "sequence",
        Kind::Context => "context",
    }
}
