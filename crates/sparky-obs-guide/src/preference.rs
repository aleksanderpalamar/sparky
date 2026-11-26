use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use sparky_config::Editor;

pub fn location() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    sparky_config::path(std::env::var("XDG_CONFIG_HOME").ok().as_deref(), &home)
}

pub fn silence(path: &Path) -> io::Result<()> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };
    let mut editor = Editor::parse(&text).map_err(io::Error::other)?;
    editor.set_flag("resources", "obs_guide", false);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("toml.new");
    fs::write(&temporary, editor.contents())?;
    fs::rename(&temporary, path)
}
