use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::{Notify, watch};

use super::{Error, Runner};
use crate::domain::Secret;

pub struct Shared {
    pub runner: Arc<dyn Runner>,
    pub gate: tokio::sync::Mutex<()>,
    pending: Mutex<HashMap<Secret, watch::Sender<bool>>>,
    changed: Notify,
}

pub struct Lease {
    shared: Arc<Shared>,
    cookie: Secret,
    pub cancellation: watch::Receiver<bool>,
}

impl Shared {
    pub fn new(runner: Arc<dyn Runner>) -> Arc<Self> {
        Arc::new(Self {
            runner,
            gate: tokio::sync::Mutex::new(()),
            pending: Mutex::default(),
            changed: Notify::new(),
        })
    }

    pub fn claim(self: &Arc<Self>, cookie: &Secret) -> Result<Lease, Error> {
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| Error::Failed("Fila de autenticação indisponível".to_owned()))?;
        if pending.len() >= 32 || pending.contains_key(cookie) {
            return Err(Error::Failed(
                "Pedido repetido ou fila de autenticação cheia".to_owned(),
            ));
        }
        let (sender, cancellation) = watch::channel(false);
        pending.insert(cookie.clone(), sender);
        Ok(Lease {
            shared: self.clone(),
            cookie: cookie.clone(),
            cancellation,
        })
    }

    pub fn cancel(&self, cookie: &Secret) {
        let sender = self
            .pending
            .lock()
            .ok()
            .and_then(|pending| pending.get(cookie).cloned());
        if let Some(sender) = sender {
            let _ = sender.send(true);
        }
    }

    pub fn cancel_all(&self) {
        let Ok(pending) = self.pending.lock() else {
            return;
        };
        for sender in pending.values() {
            let _ = sender.send(true);
        }
    }

    pub async fn idle(&self) {
        loop {
            let notified = self.changed.notified();
            if self.pending.lock().is_ok_and(|pending| pending.is_empty()) {
                return;
            }
            notified.await;
        }
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        if let Ok(mut pending) = self.shared.pending.lock() {
            pending.remove(&self.cookie);
        }
        self.shared.changed.notify_waiters();
    }
}
