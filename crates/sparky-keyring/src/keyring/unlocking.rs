use std::sync::Mutex;

use crate::keyring::{Keyring, KeyringError, Status};
use crate::ports::prompter::{Answer, Ask, Prompter};
use crate::seal::SealError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Unlocked,
    Dismissed,
}

pub fn unlock(keyring: &Mutex<Keyring>, prompter: &dyn Prompter, now: u64) -> Outcome {
    let Ok(status) = keyring.lock().map(|keyring| keyring.status()) else {
        return Outcome::Dismissed;
    };
    match status {
        Status::Unlocked => Outcome::Unlocked,
        Status::Missing => create(keyring, prompter, now),
        Status::Locked => open(keyring, prompter),
    }
}

fn create(keyring: &Mutex<Keyring>, prompter: &dyn Prompter, now: u64) -> Outcome {
    loop {
        let Answer::Password(password) = prompter.ask(Ask::Create) else {
            return Outcome::Dismissed;
        };
        if password.is_empty() {
            continue;
        }
        return settle(with(keyring, |keyring| keyring.create(password.as_bytes(), now)));
    }
}

fn open(keyring: &Mutex<Keyring>, prompter: &dyn Prompter) -> Outcome {
    let mut ask = Ask::Unlock;
    loop {
        let Answer::Password(password) = prompter.ask(ask) else {
            return Outcome::Dismissed;
        };
        match with(keyring, |keyring| keyring.unlock(password.as_bytes())) {
            Err(KeyringError::Seal(SealError::WrongPassword)) => ask = Ask::Retry,
            result => return settle(result),
        }
    }
}

fn with(keyring: &Mutex<Keyring>, action: impl FnOnce(&mut Keyring) -> Result<(), KeyringError>) -> Result<(), KeyringError> {
    match keyring.lock() {
        Ok(mut keyring) => action(&mut keyring),
        Err(_) => Err(KeyringError::Locked),
    }
}

fn settle(result: Result<(), KeyringError>) -> Outcome {
    match result {
        Ok(()) => Outcome::Unlocked,
        Err(error) => {
            eprintln!("sparky-keyring: {error}");
            Outcome::Dismissed
        }
    }
}
