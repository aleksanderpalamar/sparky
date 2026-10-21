use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex};

use zeroize::Zeroizing;

use crate::ports::prompter::{Answer, Ask, Prompter};
use crate::ports::store::VaultStore;

#[derive(Clone, Default)]
pub struct MemoryStore {
    contents: Arc<Mutex<Option<String>>>,
    failing: bool,
}

impl MemoryStore {
    pub fn failing(&self) -> Self {
        Self { contents: Arc::clone(&self.contents), failing: true }
    }

    pub fn contents(&self) -> Option<String> {
        self.contents.lock().ok().and_then(|contents| contents.clone())
    }

    pub fn put(&self, text: &str) {
        if let Ok(mut contents) = self.contents.lock() {
            *contents = Some(text.to_owned());
        }
    }
}

impl VaultStore for MemoryStore {
    fn load(&self) -> io::Result<Option<String>> {
        Ok(self.contents())
    }

    fn save(&self, contents: &str) -> io::Result<()> {
        if self.failing {
            return Err(io::Error::other("disco cheio"));
        }
        self.put(contents);
        Ok(())
    }
}

#[derive(Default)]
pub struct ScriptedPrompter {
    answers: Mutex<VecDeque<&'static str>>,
    asked: Mutex<Vec<Ask>>,
}

impl ScriptedPrompter {
    pub fn new(answers: &[&'static str]) -> Self {
        Self { answers: Mutex::new(answers.iter().copied().collect()), asked: Mutex::default() }
    }

    pub fn asked(&self) -> Vec<Ask> {
        self.asked.lock().map(|asked| asked.clone()).unwrap_or_default()
    }
}

impl Prompter for ScriptedPrompter {
    fn ask(&self, ask: Ask) -> Answer {
        if let Ok(mut asked) = self.asked.lock() {
            asked.push(ask);
        }
        let next = self.answers.lock().ok().and_then(|mut answers| answers.pop_front());
        match next {
            Some(password) => Answer::Password(Zeroizing::new(password.to_owned())),
            None => Answer::Cancelled,
        }
    }
}
