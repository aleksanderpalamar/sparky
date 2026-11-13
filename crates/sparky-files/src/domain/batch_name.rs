use std::collections::BTreeSet;

use crate::domain::naming::{split, valid};

const NUMBER: &str = "{n}";
const ARROW: &str = "->";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Numbered(String),
    Replace { from: String, to: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    Invalid(String),
    Repeated(String),
    Taken(String),
    Unchanged,
}

pub fn parse(input: &str) -> Option<Pattern> {
    if input.contains(NUMBER) {
        return Some(Pattern::Numbered(input.trim().to_owned()));
    }
    let (from, to) = input.split_once(ARROW)?;
    let from = from.trim();
    (!from.is_empty()).then(|| Pattern::Replace {
        from: from.to_owned(),
        to: to.trim().to_owned(),
    })
}

pub fn plan(names: &[String], pattern: &Pattern) -> Vec<(String, String)> {
    let width = names.len().to_string().len();
    names
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let renamed = match pattern {
                Pattern::Numbered(template) => numbered(name, template, index + 1, width),
                Pattern::Replace { from, to } => name.replace(from.as_str(), to),
            };
            (name.clone(), renamed)
        })
        .collect()
}

pub fn check(pairs: &[(String, String)], taken: impl Fn(&str) -> bool) -> Result<(), Problem> {
    if pairs.iter().all(|(old, new)| old == new) {
        return Err(Problem::Unchanged);
    }
    let sources: BTreeSet<&str> = pairs.iter().map(|(old, _)| old.as_str()).collect();
    let mut seen = BTreeSet::new();
    for (_, new) in pairs {
        if !valid(new) {
            return Err(Problem::Invalid(new.clone()));
        }
        if !seen.insert(new.as_str()) {
            return Err(Problem::Repeated(new.clone()));
        }
        if !sources.contains(new.as_str()) && taken(new) {
            return Err(Problem::Taken(new.clone()));
        }
    }
    Ok(())
}

fn numbered(name: &str, template: &str, number: usize, width: usize) -> String {
    let (_, extension) = split(name);
    let stem = template.replace(NUMBER, &format!("{number:0width$}"));
    format!("{stem}{extension}")
}
