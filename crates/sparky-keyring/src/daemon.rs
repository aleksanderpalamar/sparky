use std::fmt;
use std::sync::Arc;

use zbus::Connection;
use zbus::fdo::{RequestNameFlags, RequestNameReply};

use crate::adapters::file_store::FileStore;
use crate::adapters::prompt_process::PromptProcess;
use crate::adapters::session_end;
use crate::keyring::{Keyring, KeyringError};
use crate::seal::Kdf;
use crate::service::{self, shared::Shared};

pub const BUS_NAME: &str = "org.freedesktop.secrets";

#[derive(Debug)]
pub enum DaemonError {
    NoHome,
    NameTaken,
    Keyring(KeyringError),
    Bus(zbus::Error),
}

impl fmt::Display for DaemonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaemonError::NoHome => formatter.write_str("sem HOME nem XDG_DATA_HOME para guardar o cofre"),
            DaemonError::NameTaken => write!(formatter, "outro chaveiro já atende em {BUS_NAME} (gnome-keyring?)"),
            DaemonError::Keyring(error) => error.fmt(formatter),
            DaemonError::Bus(error) => write!(formatter, "D-Bus: {error}"),
        }
    }
}

impl From<KeyringError> for DaemonError {
    fn from(error: KeyringError) -> Self {
        DaemonError::Keyring(error)
    }
}

impl From<zbus::Error> for DaemonError {
    fn from(error: zbus::Error) -> Self {
        DaemonError::Bus(error)
    }
}

pub async fn run() -> Result<(), DaemonError> {
    let store = FileStore::from_env().ok_or(DaemonError::NoHome)?;
    let keyring = Keyring::load(Box::new(store), Kdf::STANDARD)?;
    let shared = Arc::new(Shared::new(keyring, Arc::new(PromptProcess::default())));
    let connection = zbus::connection::Builder::session()?.build().await?;
    service::serve(&connection, shared).await?;
    claim(&connection).await?;
    session_end::wait().await;
    Ok(())
}

async fn claim(connection: &Connection) -> Result<(), DaemonError> {
    match connection.request_name_with_flags(BUS_NAME, RequestNameFlags::DoNotQueue.into()).await {
        Ok(RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner) => Ok(()),
        Ok(_) | Err(zbus::Error::NameTaken) => Err(DaemonError::NameTaken),
        Err(error) => Err(error.into()),
    }
}
