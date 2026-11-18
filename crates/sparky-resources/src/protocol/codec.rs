use std::io::{self, BufRead, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::message::{Envelope, Refusal, VERSION};

pub const MAX_LINE: usize = 16 * 1024;

#[derive(Debug)]
pub enum LineError {
    Closed,
    TooLarge,
    Io(io::Error),
}

pub fn read_line(reader: &mut impl BufRead, limit: usize) -> Result<Vec<u8>, LineError> {
    let mut line = Vec::new();
    loop {
        let chunk = reader.fill_buf().map_err(LineError::Io)?;
        if chunk.is_empty() {
            return finish(line);
        }
        let end = chunk.iter().position(|byte| *byte == b'\n');
        let taken = end.map_or(chunk.len(), |index| index + 1);
        if line.len() + taken > limit + 1 {
            return Err(LineError::TooLarge);
        }
        line.extend_from_slice(&chunk[..taken]);
        reader.consume(taken);
        if end.is_some() {
            line.pop();
            return Ok(line);
        }
    }
}

fn finish(line: Vec<u8>) -> Result<Vec<u8>, LineError> {
    if line.is_empty() {
        return Err(LineError::Closed);
    }
    Ok(line)
}

pub fn decode<T: DeserializeOwned>(line: &[u8]) -> Result<T, Refusal> {
    let value: Value = serde_json::from_slice(line).map_err(|_| Refusal::Malformed)?;
    let version = value.get("version").and_then(Value::as_u64);
    if version != Some(u64::from(VERSION)) {
        return Err(Refusal::UnsupportedVersion);
    }
    let envelope: Envelope<T> = serde_json::from_value(value).map_err(|_| Refusal::Malformed)?;
    Ok(envelope.body)
}

pub fn write<T: Serialize>(writer: &mut impl Write, body: T) -> io::Result<()> {
    let mut line = serde_json::to_vec(&Envelope::current(body)).map_err(io::Error::other)?;
    line.push(b'\n');
    writer.write_all(&line)?;
    writer.flush()
}
