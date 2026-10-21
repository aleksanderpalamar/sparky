use std::collections::HashMap;
use std::sync::Arc;

use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue};
use zbus::{Connection, fdo, interface};

use crate::domain::item::NewItem;
use crate::domain::vault::Existing;
use crate::service::errors::SecretError;
use crate::service::shared::{Shared, now};
use crate::service::wire::{self, WireSecret};
use crate::service::{paths, sync};

pub const LABEL: &str = "Cofre do sparky";

pub struct Collection {
    shared: Arc<Shared>,
}

impl Collection {
    pub fn new(shared: Arc<Shared>) -> Self {
        Self { shared }
    }

    fn dates(&self) -> (u64, u64) {
        let Ok(keyring) = self.shared.keyring() else {
            return (0, 0);
        };
        keyring.vault().map_or((0, 0), |vault| (vault.created, vault.modified))
    }
}

#[interface(name = "org.freedesktop.Secret.Collection")]
impl Collection {
    async fn delete(&self) -> fdo::Result<OwnedObjectPath> {
        Err(fdo::Error::NotSupported("o cofre do sparky não pode ser apagado".to_owned()))
    }

    async fn search_items(&self, attributes: HashMap<String, String>) -> Result<Vec<OwnedObjectPath>, SecretError> {
        let found = self.shared.keyring()?.search(&attributes.into_iter().collect());
        Ok(paths::items(&[found.unlocked, found.locked].concat()))
    }

    async fn create_item(
        &self,
        properties: HashMap<String, OwnedValue>,
        secret: WireSecret,
        replace: bool,
        #[zbus(connection)] connection: &Connection,
    ) -> Result<(OwnedObjectPath, OwnedObjectPath), SecretError> {
        self.shared.session(&secret.session)?;
        let new = NewItem { label: wire::label_of(&properties), attributes: wire::attributes_of(&properties), secret: secret.into_secret() };
        let existing = match replace {
            true => Existing::Replace,
            false => Existing::Keep,
        };
        let stored = self.shared.keyring()?.change(|vault| vault.store(new, existing, now()))?;
        sync::stored(connection, &self.shared, stored).await?;
        Ok((paths::item(stored.id()), paths::none()))
    }

    #[zbus(property)]
    async fn items(&self) -> Vec<OwnedObjectPath> {
        self.shared.keyring().map(|keyring| paths::items(&keyring.ids())).unwrap_or_default()
    }

    #[zbus(property)]
    async fn label(&self) -> String {
        LABEL.to_owned()
    }

    #[zbus(property)]
    async fn locked(&self) -> bool {
        self.shared.keyring().map_or(true, |keyring| keyring.is_locked())
    }

    #[zbus(property)]
    async fn created(&self) -> u64 {
        self.dates().0
    }

    #[zbus(property)]
    async fn modified(&self) -> u64 {
        self.dates().1
    }

    #[zbus(signal)]
    pub async fn item_created(emitter: &SignalEmitter<'_>, item: ObjectPath<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn item_deleted(emitter: &SignalEmitter<'_>, item: ObjectPath<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn item_changed(emitter: &SignalEmitter<'_>, item: ObjectPath<'_>) -> zbus::Result<()>;
}
