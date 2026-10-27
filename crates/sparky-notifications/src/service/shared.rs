use crate::domain::{Center, Error, Moment, Snapshot};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::sync::watch;

pub struct Shared {
    pub images: Arc<tokio::sync::Semaphore>,
    state: Mutex<Center>,
    changes: watch::Sender<u64>,
    started: Instant,
}

impl Default for Shared {
    fn default() -> Self {
        let (changes, _) = watch::channel(0);
        Self {
            images: Arc::new(tokio::sync::Semaphore::new(2)),
            state: Mutex::default(),
            changes,
            started: Instant::now(),
        }
    }
}

impl Shared {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn now(&self) -> Moment {
        Moment {
            elapsed_ms: self.started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            utc_ms: chrono::Utc::now().timestamp_millis().max(0) as u64,
        }
    }

    pub fn read<T>(&self, read: impl FnOnce(&Center) -> T) -> Result<T, Error> {
        let center = self.state.lock().map_err(|_| Error::Poisoned)?;
        Ok(read(&center))
    }

    pub fn mutate<T>(&self, mutate: impl FnOnce(&mut Center) -> T) -> Result<T, Error> {
        let (result, revision) = {
            let mut center = self.state.lock().map_err(|_| Error::Poisoned)?;
            let before = center.snapshot_revision();
            let result = mutate(&mut center);
            (
                result,
                (before != center.snapshot_revision()).then(|| center.snapshot_revision()),
            )
        };
        if let Some(revision) = revision {
            self.changes.send_replace(revision);
        }
        Ok(result)
    }

    pub fn subscribe(&self) -> watch::Receiver<u64> {
        self.changes.subscribe()
    }
    pub fn snapshot(&self) -> Result<Snapshot, Error> {
        self.read(Center::snapshot)
    }
}
