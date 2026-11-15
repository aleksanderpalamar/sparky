mod decision;
mod filter;
pub mod wire;

#[cfg(test)]
mod decision_tests;
#[cfg(test)]
mod filter_tests;
#[cfg(test)]
mod wire_tests;

use std::path::PathBuf;

pub use decision::{Decision, decide};
pub use filter::{Filter, Rule};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Purpose {
    Open { multiple: bool, directory: bool },
    Save { name: String },
    SaveFiles { names: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub purpose: Purpose,
    pub title: String,
    pub accept: Option<String>,
    pub folder: Option<PathBuf>,
    pub filters: Vec<Filter>,
    pub filter: Option<usize>,
}

impl Request {
    pub fn accept_label(&self) -> Option<String> {
        let label = without_mnemonic(self.accept.as_deref()?);
        (!label.trim().is_empty()).then_some(label)
    }
}

fn without_mnemonic(label: &str) -> String {
    let mut shown = String::new();
    let mut letters = label.chars().peekable();
    while let Some(letter) = letters.next() {
        match (letter, letters.peek()) {
            ('_', Some('_')) => {
                shown.push('_');
                letters.next();
            }
            ('_', _) => {}
            (letter, _) => shown.push(letter),
        }
    }
    shown
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub paths: Vec<PathBuf>,
    pub filter: Option<usize>,
}
