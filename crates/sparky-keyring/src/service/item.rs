use std::collections::HashMap;
use std::sync::Arc;

use zbus::zvariant::{ObjectPath, OwnedObjectPath};
use zbus::{Connection, fdo, interface};

use crate::domain::item::{Edit, Item};
use crate::service::errors::SecretError;
use crate::service::shared::{Shared, now};
use crate::service::wire::WireSecret;
use crate::service::{paths, sync};

pub struct ItemObject {
    id: u64,
    shared: Arc<Shared>,
}

impl ItemObject {
    pub fn new(id: u64, shared: Arc<Shared>) -> Self {
        Self { id, shared }
    }

    fn read<T: Default>(&self, field: impl FnOnce(&Item) -> T) -> T {
        let Ok(keyring) = self.shared.keyring() else {
            return T::default();
        };
        keyring.item(self.id).map(field).unwrap_or_default()
    }

    fn edit(&self, edit: Edit) -> Result<(), SecretError> {
        let id = self.id;
        match self.shared.keyring()?.change(|vault| vault.edit(id, edit, now()))? {
            true => Ok(()),
            false => Err(SecretError::NoSuchObject(paths::item(id).to_string())),
        }
    }
}

#[interface(name = "org.freedesktop.Secret.Item")]
impl ItemObject {
    async fn delete(&self, #[zbus(connection)] connection: &Connection) -> Result<OwnedObjectPath, SecretError> {
        let id = self.id;
        let deleted = self.shared.keyring()?.change(|vault| vault.delete(id, now()))?;
        if !deleted {
            return Err(SecretError::NoSuchObject(paths::item(id).to_string()));
        }
        sync::deleted(connection, &self.shared, id).await?;
        Ok(paths::none())
    }

    async fn get_secret(&self, session: ObjectPath<'_>) -> Result<WireSecret, SecretError> {
        let session = self.shared.session(&session)?;
        let keyring = self.shared.keyring()?;
        Ok(WireSecret::of(session, &keyring.item(self.id)?.secret))
    }

    async fn set_secret(&self, secret: WireSecret, #[zbus(connection)] connection: &Connection) -> Result<(), SecretError> {
        self.shared.session(&secret.session)?;
        self.edit(Edit::Secret(secret.into_secret()))?;
        Ok(sync::changed(connection, self.id).await?)
    }

    #[zbus(property)]
    async fn locked(&self) -> bool {
        self.shared.keyring().map_or(true, |keyring| keyring.is_locked())
    }

    #[zbus(property)]
    async fn attributes(&self) -> HashMap<String, String> {
        self.read(|item| item.attributes.clone().into_iter().collect())
    }

    #[zbus(property)]
    async fn set_attributes(&mut self, attributes: HashMap<String, String>) -> fdo::Result<()> {
        Ok(self.edit(Edit::Attributes(attributes.into_iter().collect()))?)
    }

    #[zbus(property)]
    async fn label(&self) -> String {
        self.read(|item| item.label.clone())
    }

    #[zbus(property)]
    async fn set_label(&mut self, label: String) -> fdo::Result<()> {
        Ok(self.edit(Edit::Label(label))?)
    }

    #[zbus(property)]
    async fn created(&self) -> u64 {
        self.read(|item| item.created)
    }

    #[zbus(property)]
    async fn modified(&self) -> u64 {
        self.read(|item| item.modified)
    }
}
