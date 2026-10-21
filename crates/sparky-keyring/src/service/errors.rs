use zbus::DBusError;

use crate::keyring::KeyringError;

#[derive(Debug, DBusError)]
#[zbus(prefix = "org.freedesktop.Secret.Error")]
pub enum SecretError {
    #[zbus(error)]
    ZBus(zbus::Error),
    IsLocked(String),
    NoSession(String),
    NoSuchObject(String),
}

impl SecretError {
    pub fn failed(text: impl Into<String>) -> Self {
        SecretError::ZBus(zbus::Error::FDO(Box::new(zbus::fdo::Error::Failed(text.into()))))
    }
}

impl From<KeyringError> for SecretError {
    fn from(error: KeyringError) -> Self {
        match error {
            KeyringError::Locked | KeyringError::Missing => SecretError::IsLocked(error.to_string()),
            KeyringError::NoSuchItem => SecretError::NoSuchObject(error.to_string()),
            other => SecretError::failed(other.to_string()),
        }
    }
}

impl From<SecretError> for zbus::fdo::Error {
    fn from(error: SecretError) -> Self {
        match error {
            SecretError::ZBus(zbus::Error::FDO(inner)) => *inner,
            other => zbus::fdo::Error::Failed(other.to_string()),
        }
    }
}
