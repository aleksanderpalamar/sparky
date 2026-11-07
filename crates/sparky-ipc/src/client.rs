use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use crate::codec::{decode, encode};
use crate::error::IpcError;
use crate::message::{Event, Reply, Request};
use crate::socket;

pub struct Client {
    writer: UnixStream,
    reader: BufReader<UnixStream>,
}

impl Client {
    pub fn connect(path: &Path) -> Result<Self, IpcError> {
        let stream = UnixStream::connect(path).map_err(failure)?;
        Self::over(stream)
    }

    pub fn connect_with_timeout(path: &Path, timeout: Duration) -> Result<Self, IpcError> {
        let stream = UnixStream::connect(path).map_err(failure)?;
        stream.set_read_timeout(Some(timeout)).map_err(failure)?;
        stream.set_write_timeout(Some(timeout)).map_err(failure)?;
        Self::over(stream)
    }

    fn over(stream: UnixStream) -> Result<Self, IpcError> {
        let reader = stream.try_clone().map_err(failure)?;
        Ok(Self {
            writer: stream,
            reader: BufReader::new(reader),
        })
    }

    pub fn from_env() -> Result<Self, IpcError> {
        let path = socket::from_env().ok_or(IpcError::Unavailable)?;
        Self::connect(&path)
    }

    pub fn request(&mut self, request: &Request) -> Result<Reply, IpcError> {
        let line = encode(request)?;
        self.writer.write_all(line.as_bytes()).map_err(failure)?;
        self.read_reply()
    }

    pub fn subscribe(mut self) -> Result<Events, IpcError> {
        match self.request(&Request::Subscribe)? {
            Reply::Done => Ok(Events { client: self }),
            Reply::Error { message } => Err(IpcError::Malformed(message)),
            _ => Err(IpcError::Malformed(
                "resposta inesperada à inscrição".to_owned(),
            )),
        }
    }

    fn read_reply(&mut self) -> Result<Reply, IpcError> {
        let mut line = String::new();
        let read = self.reader.read_line(&mut line).map_err(failure)?;
        match read {
            0 => Err(IpcError::Closed),
            _ => decode(&line),
        }
    }
}

pub struct Events {
    client: Client,
}

impl Iterator for Events {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        loop {
            match self.client.read_reply().ok()? {
                Reply::Event { event } => return Some(event),
                _ => continue,
            }
        }
    }
}

fn failure(error: io::Error) -> IpcError {
    match error.kind() {
        ErrorKind::WouldBlock | ErrorKind::TimedOut => IpcError::TimedOut,
        _ => IpcError::Io(error.to_string()),
    }
}
