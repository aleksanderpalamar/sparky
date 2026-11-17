use std::collections::HashMap;

use gio::prelude::FileExt;
use zbus::zvariant::{OwnedValue, Value};

use super::options::wire_filter;
use crate::domain::choice::{Outcome, Request};

pub type Results = HashMap<String, OwnedValue>;

pub fn results(request: &Request, outcome: &Outcome) -> Results {
    let uris: Vec<String> = outcome
        .paths
        .iter()
        .map(|path| gio::File::for_path(path).uri().to_string())
        .collect();
    let mut results = Results::new();
    insert(&mut results, "uris", Value::from(uris));
    let chosen = outcome.filter.and_then(|index| request.filters.get(index));
    if let Some(filter) = chosen {
        insert(
            &mut results,
            "current_filter",
            Value::from(wire_filter(filter)),
        );
    }
    results
}

fn insert(results: &mut Results, key: &str, value: Value<'_>) {
    if let Ok(value) = OwnedValue::try_from(value) {
        results.insert(key.to_owned(), value);
    }
}
