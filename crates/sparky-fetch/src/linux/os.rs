use std::path::PathBuf;

use crate::domain::status::{Status, Value};
use crate::linux::files;
use crate::linux::host::Host;

const RELEASES: [&str; 2] = ["/etc/os-release", "/usr/lib/os-release"];
const DEFAULT: &str = "Linux";

pub fn collect(host: &dyn Host) -> Result<Value, Status> {
    let paths: Vec<PathBuf> = RELEASES.iter().map(PathBuf::from).collect();
    let (_, text) = files::first(host, &paths)?;
    Ok(Value::Text(name(&text)))
}

pub fn name(release: &str) -> String {
    ["PRETTY_NAME", "NAME"]
        .into_iter()
        .find_map(|key| variable(release, key))
        .unwrap_or_else(|| DEFAULT.to_owned())
}

fn variable(release: &str, key: &str) -> Option<String> {
    release
        .lines()
        .filter_map(|line| line.trim().split_once('='))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| unquoted(value.trim()))
        .filter(|value| !value.is_empty())
}

fn unquoted(value: &str) -> String {
    let inner = ['"', '\'']
        .into_iter()
        .find_map(|quote| value.strip_prefix(quote)?.strip_suffix(quote))
        .unwrap_or(value);
    let mut text = String::with_capacity(inner.len());
    let mut characters = inner.chars();
    while let Some(character) = characters.next() {
        match character {
            '\\' => text.extend(characters.next()),
            other => text.push(other),
        }
    }
    text
}
