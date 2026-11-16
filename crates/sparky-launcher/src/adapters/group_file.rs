use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::domain::groups::Group;
use crate::ports::GroupStore;

const STAGED: &str = "launcher-groups.new";

pub struct GroupFile {
    file: PathBuf,
}

impl GroupFile {
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }

    pub fn from_env() -> Self {
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .unwrap_or_default();
        Self::new(location(&config))
    }
}

impl GroupStore for GroupFile {
    fn load(&self) -> Result<Vec<Group>, String> {
        match fs::read_to_string(&self.file) {
            Ok(contents) => Ok(contents.lines().filter_map(parse).collect()),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error.to_string()),
        }
    }

    fn save(&self, groups: &[Group]) -> Result<(), String> {
        let parent = self.file.parent().ok_or("invalid groups directory")?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let staged = parent.join(STAGED);
        let text: String = groups.iter().map(render).collect();
        fs::write(&staged, text).map_err(|error| error.to_string())?;
        fs::rename(&staged, &self.file).map_err(|error| error.to_string())
    }
}

pub fn location(config: &Path) -> PathBuf {
    config.join("sparky/launcher-groups")
}

fn parse(line: &str) -> Option<Group> {
    let (name, apps) = line.split_once('\t').unwrap_or((line, ""));
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    Some(Group {
        name: name.to_owned(),
        apps: apps
            .split(';')
            .map(str::trim)
            .filter(|app| !app.is_empty())
            .map(str::to_owned)
            .collect(),
    })
}

fn render(group: &Group) -> String {
    let name = group.name.replace(['\t', '\n'], " ");
    format!("{name}\t{}\n", group.apps.join(";"))
}
