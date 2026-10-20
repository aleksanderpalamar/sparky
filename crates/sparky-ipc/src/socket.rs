use std::env;
use std::path::PathBuf;

pub const VARIABLE: &str = "SPARKY_SOCKET";

pub fn path_for(runtime_dir: &str, wayland_display: &str) -> PathBuf {
    PathBuf::from(runtime_dir).join(format!("sparky-{wayland_display}.sock"))
}

pub fn from_env() -> Option<PathBuf> {
    let configured = env::var(VARIABLE).ok().filter(|value| !value.trim().is_empty());
    if let Some(path) = configured {
        return Some(PathBuf::from(path));
    }
    let runtime_dir = env::var("XDG_RUNTIME_DIR").ok()?;
    let display = env::var("WAYLAND_DISPLAY").ok()?;
    Some(path_for(&runtime_dir, &display))
}

#[cfg(test)]
mod tests {
    use super::path_for;
    use std::path::PathBuf;

    #[test]
    fn o_socket_leva_o_nome_do_display() {
        assert_eq!(
            path_for("/run/user/1000", "wayland-1"),
            PathBuf::from("/run/user/1000/sparky-wayland-1.sock")
        );
    }
}
