mod channel;
mod commands;
mod driver;
mod snapshot;

use std::os::unix::net::UnixStream;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::Duration;

use iced::futures::channel::{mpsc as events, oneshot};
use iced::futures::future::BoxFuture;

use crate::domain::audio::{Audio, AudioRequest};
use crate::ports::audio::AudioControl;
use channel::{Signal, Wake};

pub struct Pulse {
    wake: Wake,
    stopped: Arc<AtomicBool>,
}

impl std::fmt::Debug for Pulse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Pulse")
    }
}

impl Pulse {
    pub fn start() -> Result<(Arc<Self>, events::UnboundedReceiver<Audio>), String> {
        let (reader, writer) = UnixStream::pair().map_err(|error| error.to_string())?;
        reader
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        writer
            .set_nonblocking(true)
            .map_err(|error| error.to_string())?;
        let (sender, receiver) = mpsc::channel();
        let wake = Wake::new(sender, writer);
        let stopped = Arc::new(AtomicBool::new(false));
        let (output, input) = events::unbounded();
        let driver_wake = wake.clone();
        let driver_stop = stopped.clone();
        std::thread::Builder::new()
            .name("sparky-audio".to_owned())
            .spawn(move || {
                driver::run(reader, receiver, driver_wake, driver_stop, output);
            })
            .map_err(|error| error.to_string())?;
        Ok((Arc::new(Self { wake, stopped }), input))
    }
}

impl AudioControl for Pulse {
    fn send(&self, request: AudioRequest) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            let (sender, receiver) = oneshot::channel();
            self.wake.send(Signal::Request(request, sender))?;
            match tokio::time::timeout(Duration::from_secs(5), receiver).await {
                Ok(Ok(result)) => result,
                Ok(Err(_)) => Err("A conexão de áudio foi encerrada".to_owned()),
                Err(_) => Err("O serviço de áudio demorou para responder".to_owned()),
            }
        })
    }
}

impl Drop for Pulse {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        let _ = self.wake.send(Signal::Shutdown);
    }
}

#[cfg(test)]
mod tests;
