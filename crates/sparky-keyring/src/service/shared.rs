use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

use zbus::zvariant::OwnedObjectPath;

use crate::keyring::Keyring;
use crate::keyring::unlocking::{self, Outcome};
use crate::ports::prompter::Prompter;
use crate::service::errors::SecretError;
use crate::service::paths;

pub struct Shared {
    keyring: Mutex<Keyring>,
    prompter: Arc<dyn Prompter>,
    gate: Mutex<()>,
    sessions: Mutex<BTreeSet<String>>,
    registered: Mutex<BTreeSet<u64>>,
    counter: AtomicU64,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Changes {
    pub added: Vec<u64>,
    pub removed: Vec<u64>,
}

impl Shared {
    pub fn new(keyring: Keyring, prompter: Arc<dyn Prompter>) -> Self {
        Self {
            keyring: Mutex::new(keyring),
            prompter,
            gate: Mutex::default(),
            sessions: Mutex::default(),
            registered: Mutex::default(),
            counter: AtomicU64::new(0),
        }
    }

    pub fn keyring(&self) -> Result<MutexGuard<'_, Keyring>, SecretError> {
        self.keyring.lock().map_err(|_| SecretError::failed("o chaveiro parou depois de um erro"))
    }

    pub fn next(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::Relaxed) + 1
    }

    pub fn open_session(&self) -> OwnedObjectPath {
        let path = paths::session(self.next());
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.insert(path.to_string());
        }
        path
    }

    pub fn close_session(&self, path: &str) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(path);
        }
    }

    pub fn session(&self, path: &str) -> Result<OwnedObjectPath, SecretError> {
        let known = self.sessions.lock().is_ok_and(|sessions| sessions.contains(path));
        match known {
            true => OwnedObjectPath::try_from(path.to_owned()).map_err(|error| SecretError::ZBus(error.into())),
            false => Err(SecretError::NoSession(format!("sessão desconhecida: {path}"))),
        }
    }

    pub fn unlock(&self) -> Outcome {
        let Ok(_gate) = self.gate.lock() else {
            return Outcome::Dismissed;
        };
        unlocking::unlock(&self.keyring, self.prompter.as_ref(), now())
    }

    pub fn reconcile(&self) -> Result<Changes, SecretError> {
        let wanted: BTreeSet<u64> = self.keyring()?.ids().into_iter().collect();
        let mut registered = self.registered.lock().map_err(|_| SecretError::failed("registro de itens perdido"))?;
        let changes = difference(&registered, &wanted);
        *registered = wanted;
        Ok(changes)
    }

    pub fn registered(&self) -> Vec<u64> {
        self.registered.lock().map(|registered| registered.iter().copied().collect()).unwrap_or_default()
    }
}

pub fn difference(registered: &BTreeSet<u64>, wanted: &BTreeSet<u64>) -> Changes {
    Changes {
        added: wanted.difference(registered).copied().collect(),
        removed: registered.difference(wanted).copied().collect(),
    }
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{Changes, difference};

    #[test]
    fn registra_os_novos_e_retira_os_que_sumiram() {
        let registrados: BTreeSet<u64> = [1, 2, 3].into();
        let desejados: BTreeSet<u64> = [2, 3, 4].into();
        assert_eq!(difference(&registrados, &desejados), Changes { added: vec![4], removed: vec![1] });
        assert_eq!(difference(&desejados, &desejados), Changes::default());
    }
}
