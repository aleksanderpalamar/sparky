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
            .map_err(|_| crate::i18n::text("audio-service-ended"))?;
        let result = self
            .socket
            .lock()
            .map_err(|_| crate::i18n::text("audio-channel-unavailable"))?
            .write_all(&[1]);
        match result {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == ErrorKind::WouldBlock => Ok(()),
            Err(error) => {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("reason", error.to_string());
                Err(crate::i18n::message("audio-notify-failed", Some(&args)))
            }
        }
    }
}
