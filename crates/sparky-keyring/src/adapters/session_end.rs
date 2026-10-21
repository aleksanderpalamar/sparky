use std::path::{Path, PathBuf};

use tokio::io::AsyncReadExt;
use tokio::net::UnixStream;

pub fn socket_path(runtime_dir: Option<&str>, display: Option<&str>) -> Option<PathBuf> {
    let display = display.filter(|display| !display.is_empty())?;
    if display.starts_with('/') {
        return Some(PathBuf::from(display));
    }
    let runtime = runtime_dir.filter(|runtime| runtime.starts_with('/'))?;
    Some(Path::new(runtime).join(display))
}

pub async fn wait() {
    let runtime = std::env::var("XDG_RUNTIME_DIR").ok();
    let display = std::env::var("WAYLAND_DISPLAY").ok();
    let Some(path) = socket_path(runtime.as_deref(), display.as_deref()) else {
        return std::future::pending().await;
    };
    let Ok(mut stream) = UnixStream::connect(&path).await else {
        return;
    };
    let mut buffer = [0u8; 64];
    while let Ok(1..) = stream.read(&mut buffer).await {}
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::socket_path;

    #[test]
    fn o_soquete_vem_do_wayland_display() {
        assert_eq!(socket_path(Some("/run/user/1000"), Some("wayland-1")), Some(PathBuf::from("/run/user/1000/wayland-1")));
        assert_eq!(socket_path(None, Some("/tmp/sparky.sock")), Some(PathBuf::from("/tmp/sparky.sock")));
        assert_eq!(socket_path(Some("/run/user/1000"), None), None);
        assert_eq!(socket_path(None, Some("wayland-1")), None);
        assert_eq!(socket_path(Some("/run/user/1000"), Some("")), None);
    }
}
