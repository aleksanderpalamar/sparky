use std::collections::HashMap;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;

use zbus::zvariant::OwnedValue;

use crate::domain::choice::{Filter, Purpose, Request, Rule};

pub type Options = HashMap<String, OwnedValue>;
pub type WireFilter = (String, Vec<(u32, String)>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Call {
    OpenFile,
    SaveFile,
    SaveFiles,
}

pub fn request(call: Call, title: &str, options: &Options) -> Request {
    let mut filters: Vec<Filter> = get::<Vec<WireFilter>>(options, "filters")
        .unwrap_or_default()
        .iter()
        .map(filter_of)
        .collect();
    let current = get::<WireFilter>(options, "current_filter").map(|wire| filter_of(&wire));
    let filter = current.and_then(|current| position(&mut filters, current));
    let (purpose, folder) = purpose(call, options);
    Request {
        purpose,
        title: title.to_owned(),
        accept: get::<String>(options, "accept_label").filter(|label| !label.is_empty()),
        folder,
        filters,
        filter,
    }
}

pub fn wire_filter(filter: &Filter) -> WireFilter {
    let rules = filter
        .rules
        .iter()
        .map(|rule| match rule {
            Rule::Glob(pattern) => (0, pattern.clone()),
            Rule::Mime(mime) => (1, mime.clone()),
        })
        .collect();
    (filter.name.clone(), rules)
}

fn purpose(call: Call, options: &Options) -> (Purpose, Option<PathBuf>) {
    let folder = get::<Vec<u8>>(options, "current_folder").and_then(path_of);
    match call {
        Call::OpenFile => (
            Purpose::Open {
                multiple: get(options, "multiple").unwrap_or(false),
                directory: get(options, "directory").unwrap_or(false),
            },
            folder,
        ),
        Call::SaveFile => save(options, folder),
        Call::SaveFiles => {
            let names = get::<Vec<Vec<u8>>>(options, "files")
                .unwrap_or_default()
                .into_iter()
                .filter_map(path_of)
                .filter_map(|path| {
                    path.file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                })
                .collect();
            (Purpose::SaveFiles { names }, folder)
        }
    }
}

fn save(options: &Options, folder: Option<PathBuf>) -> (Purpose, Option<PathBuf>) {
    let file = get::<Vec<u8>>(options, "current_file").and_then(path_of);
    let name = file
        .as_deref()
        .and_then(|file| file.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .or_else(|| get::<String>(options, "current_name"))
        .unwrap_or_default();
    let folder = file
        .as_deref()
        .and_then(|file| file.parent())
        .map(PathBuf::from)
        .or(folder);
    (Purpose::Save { name }, folder)
}

fn position(filters: &mut Vec<Filter>, current: Filter) -> Option<usize> {
    if let Some(index) = filters.iter().position(|filter| *filter == current) {
        return Some(index);
    }
    filters.is_empty().then(|| {
        filters.push(current);
        0
    })
}

fn filter_of((name, rules): &WireFilter) -> Filter {
    Filter {
        name: name.clone(),
        rules: rules
            .iter()
            .filter_map(|(kind, pattern)| match kind {
                0 => Some(Rule::Glob(pattern.clone())),
                1 => Some(Rule::Mime(pattern.clone())),
                _ => None,
            })
            .collect(),
    }
}

fn path_of(mut bytes: Vec<u8>) -> Option<PathBuf> {
    while bytes.last() == Some(&0) {
        bytes.pop();
    }
    (!bytes.is_empty()).then(|| PathBuf::from(OsString::from_vec(bytes)))
}

fn get<T: TryFrom<OwnedValue>>(options: &Options, key: &str) -> Option<T> {
    T::try_from(options.get(key)?.try_clone().ok()?).ok()
}
