use std::os::fd::OwnedFd;
use std::path::Path;
use std::process::Stdio;

use serde::Serialize;
use serde::de::DeserializeOwned;
use sparky_login_core::codec::{LIMIT, decode, encode};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::process::{Child, Command};
use zeroize::Zeroizing;

use crate::cli::WORKER;

pub struct Link {
    pub child: Child,
    pub reader: BufReader<OwnedReadHalf>,
    pub writer: OwnedWriteHalf,
}

pub async fn spawn<T: Serialize>(program: &Path, job: &T) -> Result<Link, String> {
    let (ours, theirs) =
        std::os::unix::net::UnixStream::pair().map_err(|error| error.to_string())?;
    let output = theirs.try_clone().map_err(|error| error.to_string())?;
    let child = Command::new(program)
        .arg(WORKER)
        .stdin(Stdio::from(OwnedFd::from(theirs)))
        .stdout(Stdio::from(OwnedFd::from(output)))
        .spawn()
        .map_err(|error| format!("{}: {error}", program.display()))?;
    ours.set_nonblocking(true)
        .map_err(|error| error.to_string())?;
    let (reader, mut writer) = UnixStream::from_std(ours)
        .map_err(|error| error.to_string())?
        .into_split();
    send(&mut writer, job).await?;
    Ok(Link {
        child,
        reader: BufReader::new(reader),
        writer,
    })
}

pub async fn send<T: Serialize>(writer: &mut OwnedWriteHalf, message: &T) -> Result<(), String> {
    let line = encode(message).map_err(|error| error.to_string())?;
    writer
        .write_all(line.as_bytes())
        .await
        .map_err(|error| error.to_string())
}

pub async fn next<T: DeserializeOwned>(reader: &mut BufReader<OwnedReadHalf>) -> Option<T> {
    let mut line = Zeroizing::new(String::new());
    let read = (&mut *reader)
        .take(LIMIT as u64 + 1)
        .read_line(&mut line)
        .await
        .ok()?;
    if read == 0 || read > LIMIT {
        return None;
    }
    match decode(&line) {
        Ok(message) => Some(message),
        Err(error) => {
            eprintln!("sparky-login: {error}");
            None
        }
    }
}
