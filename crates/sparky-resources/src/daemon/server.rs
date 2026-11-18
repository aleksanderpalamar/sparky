use std::fs;
use std::io::{self, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use super::service::Service;
use crate::protocol::codec::{self, LineError, MAX_LINE};
use crate::protocol::{Refusal, Reply, Request};

const TIMEOUT: Duration = Duration::from_secs(2);

pub struct Server {
    listener: UnixListener,
    path: PathBuf,
}

impl Server {
    pub fn bind(path: &Path) -> io::Result<Self> {
        release_stale(path)?;
        let listener = UnixListener::bind(path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        Ok(Self {
            listener,
            path: path.to_owned(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn serve(&self, service: &Mutex<Service>) {
        for stream in self.listener.incoming() {
            let outcome = stream.and_then(|stream| answer(&stream, service));
            if let Err(error) = outcome {
                eprintln!("sparky-resources: conexão recusada: {error}");
            }
        }
    }
}

fn release_stale(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if UnixStream::connect(path).is_ok() {
        return Err(io::Error::new(
            io::ErrorKind::AddrInUse,
            "outro sparky-resources já atende",
        ));
    }
    fs::remove_file(path)
}

fn answer(stream: &UnixStream, service: &Mutex<Service>) -> io::Result<()> {
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    let reply = match codec::read_line(&mut BufReader::new(stream), MAX_LINE) {
        Ok(line) => respond(&line, service),
        Err(LineError::TooLarge) => refused(Refusal::TooLarge),
        Err(LineError::Closed) => return Ok(()),
        Err(LineError::Io(error)) => return Err(error),
    };
    let mut writer = stream;
    codec::write(&mut writer, reply)
}

fn respond(line: &[u8], service: &Mutex<Service>) -> Reply {
    match codec::decode::<Request>(line) {
        Ok(request) => service
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .answer(request),
        Err(reason) => refused(reason),
    }
}

fn refused(reason: Refusal) -> Reply {
    Reply::Refused { reason }
}
