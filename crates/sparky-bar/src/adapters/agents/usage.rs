use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Local, NaiveDate};
use serde_json::Value;
use walkdir::WalkDir;

#[derive(Debug, Default)]
struct FileState {
    offset: u64,
    modified: Option<SystemTime>,
    tokens: u64,
    previous: u64,
    messages: HashMap<String, u64>,
}

#[derive(Debug, Default)]
pub struct Usage {
    day: Option<NaiveDate>,
    claude: HashMap<PathBuf, FileState>,
    codex: HashMap<PathBuf, FileState>,
}

impl Usage {
    pub fn today(&mut self, claude: &Path, codex: &Path) -> (u64, u64) {
        let day = Local::now().date_naive();
        if self.day != Some(day) {
            self.claude.clear();
            self.codex.clear();
            self.day = Some(day);
        }
        scan(claude, &mut self.claude, day, "\"assistant\"", claude_line);
        let mut unique = HashMap::<&str, u64>::new();
        for (id, tokens) in self.claude.values().flat_map(|state| &state.messages) {
            let known = unique.entry(id).or_default();
            *known = (*known).max(*tokens);
        }
        let claude = unique
            .values()
            .fold(0_u64, |total, tokens| total.saturating_add(*tokens));
        let codex = scan(codex, &mut self.codex, day, "\"token_count\"", codex_line);
        (claude, codex)
    }
}

fn scan(
    root: &Path,
    cache: &mut HashMap<PathBuf, FileState>,
    day: NaiveDate,
    needle: &str,
    parse: fn(&Value, NaiveDate, &mut FileState),
) -> u64 {
    let mut found = HashSet::new();
    for entry in WalkDir::new(root).follow_links(false).into_iter().flatten() {
        if !entry.file_type().is_file() || entry.path().extension().is_none_or(|ext| ext != "jsonl")
        {
            continue;
        }
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if DateTime::<Local>::from(modified).date_naive() != day {
            continue;
        }
        found.insert(path.to_path_buf());
        let state = cache.entry(path.to_path_buf()).or_default();
        if metadata.len() < state.offset
            || (metadata.len() == state.offset && state.modified != Some(modified))
        {
            *state = FileState::default();
        }
        read(path, state, day, needle, parse);
        state.modified = Some(modified);
    }
    cache.retain(|path, _| found.contains(path));
    cache
        .values()
        .fold(0_u64, |total, state| total.saturating_add(state.tokens))
}

fn read(
    path: &Path,
    state: &mut FileState,
    day: NaiveDate,
    needle: &str,
    parse: fn(&Value, NaiveDate, &mut FileState),
) {
    let Ok(mut file) = File::open(path) else {
        return;
    };
    if file.seek(SeekFrom::Start(state.offset)).is_err() {
        return;
    }
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    loop {
        line.clear();
        let Ok(bytes) = reader.read_line(&mut line) else {
            break;
        };
        if bytes == 0 || !line.ends_with('\n') {
            break;
        }
        state.offset = state.offset.saturating_add(bytes as u64);
        if !line.contains(needle) {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(&line) {
            parse(&value, day, state);
        }
    }
}

fn same_day(value: &Value, day: NaiveDate) -> bool {
    value["timestamp"]
        .as_str()
        .and_then(|stamp| DateTime::parse_from_rfc3339(stamp).ok())
        .is_some_and(|stamp| stamp.with_timezone(&Local).date_naive() == day)
}

fn claude_line(value: &Value, day: NaiveDate, state: &mut FileState) {
    if value["type"] != "assistant" || !same_day(value, day) {
        return;
    }
    let Some(id) = value["message"]["id"].as_str() else {
        return;
    };
    let usage = &value["message"]["usage"];
    let mut tokens = 0_u64;
    for key in [
        "input_tokens",
        "cache_creation_input_tokens",
        "cache_read_input_tokens",
        "output_tokens",
    ] {
        tokens = tokens.saturating_add(usage[key].as_u64().unwrap_or(0));
    }
    let previous = state.messages.entry(id.to_owned()).or_default();
    state.tokens = state
        .tokens
        .saturating_add(tokens.saturating_sub(*previous));
    *previous = (*previous).max(tokens);
}

fn codex_line(value: &Value, day: NaiveDate, state: &mut FileState) {
    if value["type"] != "event_msg" || value["payload"]["type"] != "token_count" {
        return;
    }
    let Some(total) = value["payload"]["info"]["total_token_usage"]["total_tokens"].as_u64() else {
        return;
    };
    let change = total.saturating_sub(state.previous);
    state.previous = total;
    if same_day(value, day) {
        state.tokens = state.tokens.saturating_add(change);
    }
}

#[cfg(test)]
#[path = "usage_tests.rs"]
mod tests;
