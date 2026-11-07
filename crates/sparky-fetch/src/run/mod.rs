mod pool;
#[cfg(test)]
mod run_tests;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::field::Field;
use crate::domain::report::{Entry, Report};
use crate::domain::status::{Reason, Status};
use crate::sources::Source;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub workers: usize,
    pub timeout: Duration,
}

pub fn collect(sources: Vec<Arc<dyn Source>>, wanted: &[Field], limits: Limits) -> Report {
    let jobs: Vec<Arc<dyn Source>> = sources
        .into_iter()
        .filter(|source| source.fields().iter().any(|field| wanted.contains(field)))
        .collect();
    let finished = pool::run(&jobs, wanted, limits);
    let mut found: HashMap<Field, Status> = HashMap::new();
    for (_, results) in &finished {
        for (field, status) in results {
            found.entry(*field).or_insert_with(|| status.clone());
        }
    }
    let pending: Vec<&Arc<dyn Source>> = jobs
        .iter()
        .enumerate()
        .filter(|(index, _)| !finished.iter().any(|(done, _)| done == index))
        .map(|(_, source)| source)
        .collect();
    let entries = wanted
        .iter()
        .map(|field| Entry::new(*field, outcome(*field, &found, &pending)))
        .collect();
    Report { entries }
}

fn outcome(field: Field, found: &HashMap<Field, Status>, pending: &[&Arc<dyn Source>]) -> Status {
    if let Some(status) = found.get(&field) {
        return status.clone();
    }
    match pending
        .iter()
        .any(|source| source.fields().contains(&field))
    {
        true => Status::TimedOut,
        false => Status::unavailable(Reason::Uncollected),
    }
}
