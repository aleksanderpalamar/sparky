use std::io::{self, BufReader};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use super::codec::{self, LineError, MAX_LINE};
use super::message::{Refusal, Reply, Request};

const TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug)]
pub enum ClientError {
    Unavailable(io::Error),
    Closed,
    TooLarge,
    Invalid(Refusal),
}

pub fn ask(path: &Path, request: Request) -> Result<Reply, ClientError> {
    let stream = UnixStream::connect(path).map_err(ClientError::Unavailable)?;
    stream
        .set_read_timeout(Some(TIMEOUT))
        .map_err(ClientError::Unavailable)?;
    stream
        .set_write_timeout(Some(TIMEOUT))
        .map_err(ClientError::Unavailable)?;
    let mut writer = &stream;
    codec::write(&mut writer, request).map_err(ClientError::Unavailable)?;
    let line =
        codec::read_line(&mut BufReader::new(&stream), MAX_LINE).map_err(|error| match error {
            LineError::Closed => ClientError::Closed,
            LineError::TooLarge => ClientError::TooLarge,
            LineError::Io(error) => ClientError::Unavailable(error),
        })?;
    codec::decode(&line).map_err(ClientError::Invalid)
}
