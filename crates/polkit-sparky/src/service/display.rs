use std::path::{Path, PathBuf};

use tokio::io::AsyncReadExt;
use tokio::net::UnixStream;

fn path(runtime: Option<&str>, display: Option<&str>) -> Option<PathBuf> {
    let display = display.filter(|value| !value.is_empty())?;
    if display.starts_with('/') {
        return Some(PathBuf::from(display));
    }
    let runtime = runtime.filter(|value| value.starts_with('/'))?;
    Some(Path::new(runtime).join(display))
}

async fn disconnected(path: &Path) {
    let Ok(mut stream) = UnixStream::connect(path).await else {
        return;
    };
    let mut bytes = [0; 64];
    while let Ok(1..) = stream.read(&mut bytes).await {}
}

pub async fn closed() {
    let runtime = std::env::var("XDG_RUNTIME_DIR").ok();
    let display = std::env::var("WAYLAND_DISPLAY").ok();
    match path(runtime.as_deref(), display.as_deref()) {
        Some(path) => disconnected(&path).await,
        None => std::future::pending().await,
    }
}

#[cfg(test)]
mod tests {
    use super::{disconnected, path};

    #[tokio::test]
    async fn fim_do_compositor_encerra_a_espera_sem_abrir_janelas() {
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("wayland");
        let server = tokio::net::UnixListener::bind(&socket).unwrap();
        let close = async {
            drop(server.accept().await.unwrap());
        };
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            tokio::join!(disconnected(&socket), close);
        })
        .await
        .unwrap();
    }

    #[test]
    fn soquete_usa_o_display_da_sessao() {
        assert_eq!(
            path(Some("/run/user/1000"), Some("wayland-1")),
            Some("/run/user/1000/wayland-1".into())
        );
        assert_eq!(
            path(None, Some("/tmp/wayland")),
            Some("/tmp/wayland".into())
        );
        assert_eq!(path(None, Some("wayland-1")), None);
        assert_eq!(path(Some("relativo"), Some("wayland-1")), None);
        assert_eq!(path(Some("/tmp"), Some("")), None);
    }
}
