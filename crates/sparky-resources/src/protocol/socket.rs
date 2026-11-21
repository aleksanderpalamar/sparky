use std::path::PathBuf;

pub const VARIABLE: &str = "SPARKY_RESOURCES_SOCKET";
const NAME: &str = "sparky-resources.sock";

pub fn path_from(configured: Option<String>, runtime_dir: Option<String>) -> Option<PathBuf> {
    if let Some(path) = configured.filter(|value| !value.trim().is_empty()) {
        return Some(PathBuf::from(path));
    }
    let runtime_dir = runtime_dir.filter(|value| value.starts_with('/'))?;
    Some(PathBuf::from(runtime_dir).join(NAME))
}

pub fn from_env() -> Option<PathBuf> {
    path_from(
        std::env::var(VARIABLE).ok(),
        std::env::var("XDG_RUNTIME_DIR").ok(),
    )
}
