use std::os::linux::net::SocketAddrExt;
use std::os::unix::net::{SocketAddr, UnixDatagram};

pub fn notify(state: &str) -> bool {
    let Ok(path) = std::env::var("NOTIFY_SOCKET") else {
        return false;
    };
    send(&path, state).is_ok()
}

fn send(path: &str, state: &str) -> std::io::Result<()> {
    let socket = UnixDatagram::unbound()?;
    let address = match path.strip_prefix('@') {
        Some(name) => SocketAddr::from_abstract_name(name.as_bytes())?,
        None => SocketAddr::from_pathname(path)?,
    };
    socket.send_to_addr(state.as_bytes(), &address).map(drop)
}

#[cfg(test)]
mod tests {
    use std::os::unix::net::UnixDatagram;

    use super::send;

    #[test]
    fn avisa_o_systemd_por_socket_de_datagrama() {
        let Ok(directory) = tempfile::tempdir() else {
            return;
        };
        let path = directory.path().join("notify");
        let Ok(listener) = UnixDatagram::bind(&path) else {
            return;
        };
        assert!(send(&path.to_string_lossy(), "WATCHDOG=1").is_ok());
        let mut buffer = [0; 32];
        let read = listener.recv(&mut buffer).unwrap_or_default();
        assert_eq!(&buffer[..read], b"WATCHDOG=1");
    }
}
