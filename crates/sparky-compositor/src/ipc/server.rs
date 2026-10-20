use std::env;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;

use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{Interest, Mode, PostAction};
use sparky_ipc::{WorkspaceInfo, socket};

use crate::state::Sparky;

pub struct IpcServer {
    pub path: PathBuf,
    pub subscribers: Vec<UnixStream>,
    pub last: Vec<WorkspaceInfo>,
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.path)
            && error.kind() != ErrorKind::NotFound
        {
            tracing::warn!("socket do sparky não removido: {error}");
        }
    }
}

impl Sparky {
    pub fn start_ipc(&mut self) {
        let Ok(runtime) = env::var("XDG_RUNTIME_DIR") else {
            tracing::warn!("sem XDG_RUNTIME_DIR, o sparky fica sem IPC");
            return;
        };
        let path = socket::path_for(&runtime, &self.socket_name.to_string_lossy());
        if path.exists() && fs::remove_file(&path).is_err() {
            tracing::warn!("socket antigo do sparky não removido: {}", path.display());
        }
        let listener = match UnixListener::bind(&path).and_then(|listener| listener.set_nonblocking(true).map(|()| listener)) {
            Ok(listener) => listener,
            Err(error) => {
                tracing::warn!("IPC do sparky não abriu: {error}");
                return;
            }
        };
        let inserted = self
            .loop_handle
            .insert_source(Generic::new(listener, Interest::READ, Mode::Level), |_, listener, state| {
                while let Ok((stream, _)) = listener.accept() {
                    state.watch_connection(stream);
                }
                Ok(PostAction::Continue)
            });
        match inserted {
            Ok(_) => {
                self.ipc = Some(IpcServer {
                    path,
                    subscribers: Vec::new(),
                    last: Vec::new(),
                })
            }
            Err(error) => tracing::warn!("IPC do sparky sem fonte de eventos: {error}"),
        }
    }

    pub fn ipc_path(&self) -> Option<PathBuf> {
        self.ipc.as_ref().map(|server| server.path.clone())
    }
}
