use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use sparky_ipc::{Client, GlassArea, IpcError, Request, socket};

use crate::app::surfaces::DOCK;
use crate::ports::GlassBoard;

const TIMEOUT: Duration = Duration::from_secs(1);

pub struct IpcBoard {
    sender: Sender<GlassArea>,
}

impl IpcBoard {
    pub fn start() -> Self {
        let (sender, receiver) = channel();
        std::thread::spawn(move || deliver(&receiver));
        Self { sender }
    }
}

impl GlassBoard for IpcBoard {
    fn announce(&self, area: GlassArea) {
        if self.sender.send(area).is_err() {
            eprintln!("sparky-dock: entrega da área do vidro encerrada");
        }
    }
}

fn deliver(receiver: &Receiver<GlassArea>) {
    let mut client: Option<Client> = None;
    while let Ok(first) = receiver.recv() {
        let latest = receiver.try_iter().last().unwrap_or(first);
        if let Err(error) = send(&mut client, latest) {
            client = None;
            eprintln!("sparky-dock: área do vidro não informada: {error}");
        }
    }
}

fn send(client: &mut Option<Client>, area: GlassArea) -> Result<(), IpcError> {
    let connected = match client.take() {
        Some(connected) => connected,
        None => Client::connect_with_timeout(
            &socket::from_env().ok_or(IpcError::Unavailable)?,
            TIMEOUT,
        )?,
    };
    let request = Request::Glass {
        namespace: DOCK.to_owned(),
        area,
    };
    let mut connected = connected;
    connected.request(&request)?;
    *client = Some(connected);
    Ok(())
}
