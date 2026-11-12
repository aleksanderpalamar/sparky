use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::PathBuf;

use crate::domain::transfer::{Action, Step};

const DEPTH: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrashEntry {
    pub root: PathBuf,
    pub topdir: Option<PathBuf>,
    pub name: OsString,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Record {
    Transferred { action: Action, steps: Vec<Step> },
    Renamed { from: PathBuf, to: PathBuf },
    Trashed(Vec<TrashEntry>),
    Created(PathBuf),
}

#[derive(Debug, Clone, Default)]
pub struct Journal {
    records: VecDeque<Record>,
}

impl Journal {
    pub fn push(&mut self, record: Record) {
        if self.records.len() == DEPTH {
            self.records.pop_front();
        }
        self.records.push_back(record);
    }

    pub fn pop(&mut self) -> Option<Record> {
        self.records.pop_back()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

pub fn reversed(steps: &[Step]) -> Vec<Step> {
    steps
        .iter()
        .rev()
        .map(|step| Step {
            source: step.target.clone(),
            target: step.source.clone(),
            action: Action::Move,
            replace: false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{DEPTH, Journal, Record, reversed};
    use crate::domain::transfer::{Action, Step};

    fn criado(indice: usize) -> Record {
        Record::Created(PathBuf::from(format!("/h/{indice}")))
    }

    #[test]
    fn desfazer_segue_a_ordem_inversa_e_guarda_no_maximo_o_limite() {
        let mut journal = Journal::default();
        for indice in 0..DEPTH + 5 {
            journal.push(criado(indice));
        }
        assert_eq!(journal.len(), DEPTH);
        assert_eq!(journal.pop(), Some(criado(DEPTH + 4)));
        assert_eq!(journal.pop(), Some(criado(DEPTH + 3)));
    }

    #[test]
    fn mover_de_volta_inverte_origem_destino_e_ordem() {
        let passos = vec![
            Step {
                source: PathBuf::from("/a/1"),
                target: PathBuf::from("/b/1"),
                action: Action::Move,
                replace: false,
            },
            Step {
                source: PathBuf::from("/a/2"),
                target: PathBuf::from("/b/2"),
                action: Action::Move,
                replace: false,
            },
        ];
        let volta = reversed(&passos);
        assert_eq!(volta[0].source, PathBuf::from("/b/2"));
        assert_eq!(volta[0].target, PathBuf::from("/a/2"));
        assert_eq!(volta[1].target, PathBuf::from("/a/1"));
    }
}
