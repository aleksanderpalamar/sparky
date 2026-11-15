use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};

use super::{Filter, Outcome, Purpose, Request, Rule};

pub fn encode_request(request: &Request) -> String {
    let mut fields = Map::new();
    let (kind, extra) = match &request.purpose {
        Purpose::Open {
            multiple,
            directory,
        } => (
            "open",
            json!({"multiple": multiple, "directory": directory}),
        ),
        Purpose::Save { name } => ("save", json!({"name": name})),
        Purpose::SaveFiles { names } => ("save-files", json!({"names": names})),
    };
    fields.insert("purpose".to_owned(), json!(kind));
    if let Value::Object(extra) = extra {
        fields.extend(extra);
    }
    fields.insert("title".to_owned(), json!(request.title));
    fields.insert("accept".to_owned(), json!(request.accept));
    fields.insert(
        "folder".to_owned(),
        request.folder.as_deref().map_or(Value::Null, bytes),
    );
    fields.insert(
        "filters".to_owned(),
        request.filters.iter().map(filter).collect(),
    );
    fields.insert("filter".to_owned(), json!(request.filter));
    Value::Object(fields).to_string()
}

pub fn decode_request(text: &str) -> Option<Request> {
    let value: Value = serde_json::from_str(text).ok()?;
    let purpose = match value.get("purpose")?.as_str()? {
        "open" => Purpose::Open {
            multiple: value.get("multiple")?.as_bool()?,
            directory: value.get("directory")?.as_bool()?,
        },
        "save" => Purpose::Save {
            name: value.get("name")?.as_str()?.to_owned(),
        },
        "save-files" => Purpose::SaveFiles {
            names: strings(value.get("names")?)?,
        },
        _ => return None,
    };
    let filters: Vec<Filter> = value
        .get("filters")?
        .as_array()?
        .iter()
        .map(filter_of)
        .collect::<Option<_>>()?;
    let filter = value
        .get("filter")
        .and_then(Value::as_u64)
        .map(|index| index as usize);
    Some(Request {
        purpose,
        title: value.get("title")?.as_str()?.to_owned(),
        accept: value
            .get("accept")
            .and_then(Value::as_str)
            .map(str::to_owned),
        folder: value.get("folder").and_then(path_of),
        filter: filter.filter(|index| *index < filters.len()),
        filters,
    })
}

pub fn encode_outcome(outcome: &Outcome) -> String {
    let paths: Vec<Value> = outcome.paths.iter().map(|path| bytes(path)).collect();
    json!({"paths": paths, "filter": outcome.filter}).to_string()
}

pub fn decode_outcome(text: &str) -> Option<Outcome> {
    let value: Value = serde_json::from_str(text.trim()).ok()?;
    let paths = value
        .get("paths")?
        .as_array()?
        .iter()
        .map(path_of)
        .collect::<Option<Vec<_>>>()?;
    Some(Outcome {
        paths,
        filter: value
            .get("filter")
            .and_then(Value::as_u64)
            .map(|index| index as usize),
    })
}

fn filter(filter: &Filter) -> Value {
    let rules: Vec<Value> = filter
        .rules
        .iter()
        .map(|rule| match rule {
            Rule::Glob(pattern) => json!({"glob": pattern}),
            Rule::Mime(mime) => json!({"mime": mime}),
        })
        .collect();
    json!({"name": filter.name, "rules": rules})
}

fn filter_of(value: &Value) -> Option<Filter> {
    let rules = value
        .get("rules")?
        .as_array()?
        .iter()
        .map(|rule| match (rule.get("glob"), rule.get("mime")) {
            (Some(Value::String(pattern)), _) => Some(Rule::Glob(pattern.clone())),
            (_, Some(Value::String(mime))) => Some(Rule::Mime(mime.clone())),
            _ => None,
        })
        .collect::<Option<_>>()?;
    Some(Filter {
        name: value.get("name")?.as_str()?.to_owned(),
        rules,
    })
}

fn strings(value: &Value) -> Option<Vec<String>> {
    value
        .as_array()?
        .iter()
        .map(|item| item.as_str().map(str::to_owned))
        .collect()
}

fn bytes(path: &Path) -> Value {
    json!(path.as_os_str().as_bytes())
}

fn path_of(value: &Value) -> Option<PathBuf> {
    let bytes = value
        .as_array()?
        .iter()
        .map(|byte| byte.as_u64().and_then(|byte| u8::try_from(byte).ok()))
        .collect::<Option<Vec<u8>>>()?;
    Some(PathBuf::from(OsString::from_vec(bytes)))
}
