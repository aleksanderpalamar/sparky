use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use zbus::zvariant::{ObjectPath, OwnedFd, OwnedValue};
use zbus::{Connection, interface};
use zeroize::Zeroizing;

use crate::domain::item::{Attributes, NewItem};
use crate::domain::secret::Secret;
use crate::domain::vault::Existing;
use crate::keyring::unlocking::Outcome;
use crate::seal;
use crate::service::errors::SecretError;
use crate::service::shared::{Shared, now};
use crate::service::sync;

pub const PATH: &str = "/org/freedesktop/portal/desktop";
const SECRET_SIZE: usize = 64;
const SCHEMA: &str = "org.freedesktop.portal.Secret";
const VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Response {
    Success = 0,
    Cancelled = 1,
    Failed = 2,
}

type Results = HashMap<String, OwnedValue>;

pub struct Portal {
    shared: Arc<Shared>,
}

impl Portal {
    pub fn new(shared: Arc<Shared>) -> Self {
        Self { shared }
    }

    fn app_secret(&self, app_id: &str) -> Result<Zeroizing<Vec<u8>>, SecretError> {
        let query = attributes(app_id);
        let mut keyring = self.shared.keyring()?;
        if let Some(id) = keyring.search(&query).unlocked.first() {
            return Ok(keyring.item(*id)?.secret.value.clone());
        }
        let value = Zeroizing::new(seal::random::<SECRET_SIZE>().map_err(|error| SecretError::failed(error.to_string()))?.to_vec());
        let secret = Secret::new(value.to_vec(), "application/octet-stream");
        let new = NewItem { label: format!("Segredo do aplicativo {app_id}"), attributes: query, secret };
        keyring.change(|vault| vault.store(new, Existing::Keep, now()))?;
        Ok(value)
    }
}

#[interface(name = "org.freedesktop.impl.portal.Secret")]
impl Portal {
    async fn retrieve_secret(
        &self,
        _handle: ObjectPath<'_>,
        app_id: &str,
        fd: OwnedFd,
        _options: Results,
        #[zbus(connection)] connection: &Connection,
    ) -> (u32, Results) {
        let asking = Arc::clone(&self.shared);
        let outcome = tokio::task::spawn_blocking(move || asking.unlock()).await.unwrap_or(Outcome::Dismissed);
        if outcome == Outcome::Dismissed {
            return (Response::Cancelled as u32, Results::new());
        }
        let response = match self.app_secret(app_id) {
            Ok(secret) => deliver(fd, &secret),
            Err(error) => {
                eprintln!("sparky-keyring: segredo de {app_id} indisponível: {error}");
                Response::Failed
            }
        };
        if let Err(error) = sync::refresh(connection, &self.shared).await {
            eprintln!("sparky-keyring: {error}");
        }
        (response as u32, Results::new())
    }

    #[zbus(property, name = "version")]
    async fn version(&self) -> u32 {
        VERSION
    }
}

fn attributes(app_id: &str) -> Attributes {
    [("app_id", app_id), ("xdg:schema", SCHEMA)].iter().map(|(name, value)| (name.to_string(), value.to_string())).collect()
}

fn deliver(fd: OwnedFd, secret: &[u8]) -> Response {
    let mut file = File::from(std::os::fd::OwnedFd::from(fd));
    match file.write_all(secret) {
        Ok(()) => Response::Success,
        Err(error) => {
            eprintln!("sparky-keyring: não deu para entregar o segredo: {error}");
            Response::Failed
        }
    }
}
