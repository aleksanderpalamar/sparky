use std::time::Duration;

use sparky_ipc::{Client, GlassArea, Request, socket};

use crate::app::surfaces::DOCK;
use crate::ports::GlassBoard;

const TIMEOUT: Duration = Duration::from_secs(1);

pub struct IpcBoard;

impl GlassBoard for IpcBoard {
    fn announce(&self, area: GlassArea) {
        std::thread::spawn(move || {
            let Some(path) = socket::from_env() else {
                return;
            };
            let request = Request::Glass {
                namespace: DOCK.to_owned(),
                area,
            };
            let sent = Client::connect_with_timeout(&path, TIMEOUT)
                .and_then(|mut client| client.request(&request));
            if let Err(error) = sent {
                eprintln!("sparky-dock: área do vidro não informada: {error}");
            }
        });
    }
}
