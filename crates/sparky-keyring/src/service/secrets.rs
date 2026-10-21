use std::collections::HashMap;
use std::sync::Arc;

use zbus::object_server::ObjectServer;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Str, Value};
use zbus::{Connection, fdo, interface};

use crate::service::errors::SecretError;
use crate::service::paths;
use crate::service::prompt::Prompt;
use crate::service::session::Session;
use crate::service::shared::Shared;
use crate::service::sync;
use crate::service::wire::WireSecret;

pub const PLAIN: &str = "plain";

type Paths = Vec<OwnedObjectPath>;

pub struct Service {
    shared: Arc<Shared>,
}

impl Service {
    pub fn new(shared: Arc<Shared>) -> Self {
        Self { shared }
    }
}

#[interface(name = "org.freedesktop.Secret.Service")]
impl Service {
    async fn open_session(
        &self,
        algorithm: &str,
        _input: Value<'_>,
        #[zbus(object_server)] server: &ObjectServer,
    ) -> fdo::Result<(OwnedValue, OwnedObjectPath)> {
        if algorithm != PLAIN {
            return Err(fdo::Error::NotSupported(format!("algoritmo não suportado: {algorithm}")));
        }
        let path = self.shared.open_session();
        server.at(&path, Session::new(Arc::clone(&self.shared), path.clone())).await?;
        Ok((OwnedValue::from(Str::from("")), path))
    }

    async fn create_collection(&self, _properties: HashMap<String, OwnedValue>, _alias: &str) -> (OwnedObjectPath, OwnedObjectPath) {
        (paths::collection(), paths::none())
    }

    async fn search_items(&self, attributes: HashMap<String, String>) -> Result<(Paths, Paths), SecretError> {
        let found = self.shared.keyring()?.search(&attributes.into_iter().collect());
        Ok((paths::items(&found.unlocked), paths::items(&found.locked)))
    }

    async fn unlock(&self, objects: Paths, #[zbus(object_server)] server: &ObjectServer) -> Result<(Paths, OwnedObjectPath), SecretError> {
        let locked = self.shared.keyring()?.is_locked();
        if !locked {
            return Ok((objects, paths::none()));
        }
        let path = paths::prompt(self.shared.next());
        server.at(&path, Prompt::new(Arc::clone(&self.shared), path.clone(), objects)).await?;
        Ok((Vec::new(), path))
    }

    async fn lock(&self, objects: Paths, #[zbus(connection)] connection: &Connection) -> Result<(Paths, OwnedObjectPath), SecretError> {
        self.shared.keyring()?.lock();
        sync::refresh(connection, &self.shared).await?;
        Ok((objects, paths::none()))
    }

    async fn get_secrets(&self, items: Paths, session: ObjectPath<'_>) -> Result<HashMap<OwnedObjectPath, WireSecret>, SecretError> {
        let session = self.shared.session(&session)?;
        let keyring = self.shared.keyring()?;
        let found = items.into_iter().filter_map(|path| {
            let item = keyring.item(paths::item_id(&path)?).ok()?;
            Some((path, WireSecret::of(session.clone(), &item.secret)))
        });
        Ok(found.collect())
    }

    async fn read_alias(&self, name: &str) -> OwnedObjectPath {
        paths::alias(name)
    }

    async fn set_alias(&self, name: &str, collection: ObjectPath<'_>) -> fdo::Result<()> {
        match (paths::alias(name) == paths::collection(), paths::is_collection(&collection)) {
            (true, true) => Ok(()),
            _ => Err(fdo::Error::NotSupported("o sparky tem um só cofre".to_owned())),
        }
    }

    #[zbus(property)]
    async fn collections(&self) -> Vec<OwnedObjectPath> {
        vec![paths::collection()]
    }
}
