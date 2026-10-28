use std::io::{ErrorKind, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex, mpsc};

use iced::futures::channel::oneshot;

use crate::domain::audio::AudioRequest;

pub type Reply = oneshot::Sender<Result<(), String>>;

pub enum Signal {
    State,
    Refresh,
    Request(AudioRequest, Reply),
    Shutdown,
}

#[derive(Clone)]
pub struct Wake {
    sender: mpsc::Sender<Signal>,
    socket: Arc<Mutex<UnixStream>>,
}

impl Wake {
    pub fn new(sender: mpsc::Sender<Signal>, socket: UnixStream) -> Self {
        Self {
            sender,
            socket: Arc::new(Mutex::new(socket)),
        }
    }

    pub fn send(&self, signal: Signal) -> Result<(), String> {
        self.sender
            .send(signal)
            .map_err(|_| "O serviço de áudio foi encerrado".to_owned())?;
        let result = self
            .socket
            .lock()
            .map_err(|_| "O canal de áudio está indisponível".to_owned())?
            .write_all(&[1]);
        match result {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::WouldBlock => Ok(()),
            Err(error) => Err(format!(
                "Não foi possível avisar o serviço de áudio: {error}"
            )),
        }
    }
}
