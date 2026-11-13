use std::io::{self, Read};
use std::path::PathBuf;

use serde_json::Value;

use crate::adapters::clipboard_server::serve;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    Text(String),
    File(PathBuf),
}

pub fn run_from_stdin() -> i32 {
    let mut input = String::new();
    let entries = io::stdin()
        .read_to_string(&mut input)
        .ok()
        .and_then(|_| parse(&input));
    match entries.map(serve) {
        Some(Ok(())) => 0,
        _ => 1,
    }
}

pub fn parse(input: &str) -> Option<Vec<(String, Content)>> {
    let Value::Array(items) = serde_json::from_str(input).ok()? else {
        return None;
    };
    items
        .iter()
        .map(|item| {
            let mime = item.get("mime")?.as_str()?.to_owned();
            let content = match (item.get("text"), item.get("file")) {
                (Some(text), _) => Content::Text(text.as_str()?.to_owned()),
                (None, Some(file)) => Content::File(PathBuf::from(file.as_str()?)),
                (None, None) => return None,
            };
            Some((mime, content))
        })
        .collect()
}
