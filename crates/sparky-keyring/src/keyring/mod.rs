pub mod unlocking;

#[cfg(test)]
mod keyring_tests;
#[cfg(test)]
mod unlocking_tests;

use std::{fmt, io};

use crate::domain::index::{self, Entry, index_of};
use crate::domain::item::{Attributes, Item};
use crate::domain::vault::Vault;
use crate::ports::store::VaultStore;
use crate::seal::{self, Kdf, SealError, Sealing};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Missing,
    Locked,
    Unlocked,
}

enum State {
    Missing,
    Locked(Vec<Entry>),
    Unlocked { vault: Vault, sealing: Sealing },
}

#[derive(Debug)]
pub enum KeyringError {
    Missing,
    Exists,
    Locked,
    NoSuchItem,
    Seal(SealError),
    Store(io::Error),
}

impl fmt::Display for KeyringError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyringError::Missing => formatter.write_str("o cofre ainda não foi criado"),
            KeyringError::Exists => formatter.write_str("o cofre já existe"),
            KeyringError::Locked => formatter.write_str("o cofre está trancado"),
            KeyringError::NoSuchItem => formatter.write_str("o item não existe"),
            KeyringError::Seal(error) => error.fmt(formatter),
            KeyringError::Store(error) => write!(formatter, "não deu para gravar o cofre: {error}"),
        }
    }
}

impl std::error::Error for KeyringError {}

impl From<SealError> for KeyringError {
    fn from(error: SealError) -> Self {
        KeyringError::Seal(error)
    }
}

impl From<io::Error> for KeyringError {
    fn from(error: io::Error) -> Self {
        KeyringError::Store(error)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Found {
    pub unlocked: Vec<u64>,
    pub locked: Vec<u64>,
}

pub struct Keyring {
    state: State,
    store: Box<dyn VaultStore>,
    kdf: Kdf,
}

impl Keyring {
    pub fn load(store: Box<dyn VaultStore>, kdf: Kdf) -> Result<Self, KeyringError> {
        let state = match store.load()? {
            None => State::Missing,
            Some(contents) => State::Locked(seal::header(&contents)?.index),
        };
        Ok(Self { state, store, kdf })
    }

    pub fn status(&self) -> Status {
        match self.state {
            State::Missing => Status::Missing,
            State::Locked(_) => Status::Locked,
            State::Unlocked { .. } => Status::Unlocked,
        }
    }

    pub fn is_locked(&self) -> bool {
        self.status() != Status::Unlocked
    }

    pub fn create(&mut self, password: &[u8], now: u64) -> Result<(), KeyringError> {
        if !matches!(self.state, State::Missing) {
            return Err(KeyringError::Exists);
        }
        let sealing = Sealing::create(password, self.kdf)?;
        let vault = Vault::new(now);
        self.store.save(&sealing.seal(&vault)?)?;
        self.state = State::Unlocked { vault, sealing };
        Ok(())
    }

    pub fn unlock(&mut self, password: &[u8]) -> Result<(), KeyringError> {
        match self.state {
            State::Unlocked { .. } => return Ok(()),
            State::Missing => return Err(KeyringError::Missing),
            State::Locked(_) => {}
        }
        let contents = self.store.load()?.ok_or(KeyringError::Missing)?;
        let (vault, sealing) = seal::open(&contents, password)?;
        self.state = State::Unlocked { vault, sealing };
        Ok(())
    }

    pub fn lock(&mut self) {
        if let State::Unlocked { vault, .. } = &self.state {
            self.state = State::Locked(index_of(vault));
        }
    }

    pub fn ids(&self) -> Vec<u64> {
        match &self.state {
            State::Missing => Vec::new(),
            State::Locked(entries) => entries.iter().map(|entry| entry.id).collect(),
            State::Unlocked { vault, .. } => vault.items().iter().map(|item| item.id).collect(),
        }
    }

    pub fn search(&self, query: &Attributes) -> Found {
        match &self.state {
            State::Missing => Found::default(),
            State::Locked(entries) => Found { unlocked: Vec::new(), locked: index::search(entries, query) },
            State::Unlocked { vault, .. } => Found { unlocked: vault.search(query), locked: Vec::new() },
        }
    }

    pub fn vault(&self) -> Result<&Vault, KeyringError> {
        match &self.state {
            State::Unlocked { vault, .. } => Ok(vault),
            _ => Err(KeyringError::Locked),
        }
    }

    pub fn item(&self, id: u64) -> Result<&Item, KeyringError> {
        self.vault()?.item(id).ok_or(KeyringError::NoSuchItem)
    }

    pub fn change<T>(&mut self, edit: impl FnOnce(&mut Vault) -> T) -> Result<T, KeyringError> {
        let State::Unlocked { vault, sealing } = &mut self.state else {
            return Err(KeyringError::Locked);
        };
        let mut draft = vault.clone();
        let result = edit(&mut draft);
        self.store.save(&sealing.seal(&draft)?)?;
        *vault = draft;
        Ok(result)
    }
}
