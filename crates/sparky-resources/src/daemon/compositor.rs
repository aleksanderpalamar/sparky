use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use sparky_ipc::{Client, IpcError, Reply, Request, WindowInfo};

use crate::context::Window;

const RECONNECT: Duration = Duration::from_secs(5);
const TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Default)]
pub struct Windows {
    current: Arc<Mutex<Option<Vec<Window>>>>,
}

impl Windows {
    pub fn snapshot(&self) -> Option<Vec<Window>> {
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn store(&self, windows: Option<Vec<Window>>) {
        let mut current = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        *current = windows;
    }
}

pub fn follow(path: PathBuf, windows: Windows) {
    thread::spawn(move || {
        loop {
            let ended = watch(&path, &windows);
            windows.store(None);
            if let Err(error) = ended {
                eprintln!("sparky-resources: sem contexto do compositor: {error}");
            }
            thread::sleep(RECONNECT);
        }
    });
}

fn watch(path: &Path, windows: &Windows) -> Result<(), IpcError> {
    let mut asking = Client::connect_with_timeout(path, TIMEOUT)?;
    windows.store(Some(listed(&mut asking)?));
    for _ in Client::connect(path)?.watch_windows()? {
        windows.store(Some(listed(&mut asking)?));
    }
    Ok(())
}

fn listed(client: &mut Client) -> Result<Vec<Window>, IpcError> {
    match client.request(&Request::Windows)? {
        Reply::Windows { windows } => Ok(windows.into_iter().map(window).collect()),
        _ => Err(IpcError::Malformed(
            "o compositor não informou as janelas".to_owned(),
        )),
    }
}

fn window(info: WindowInfo) -> Window {
    Window {
        app_id: info.app_id,
        pid: info.pid,
        focused: info.focused,
        fullscreen: info.fullscreen,
    }
}
