use std::fmt;
use std::io::{self, BufRead, Read, Write};

use serde::Serialize;
use serde::de::DeserializeOwned;
use zeroize::Zeroizing;

pub const LIMIT: usize = 64 * 1024;

#[derive(Debug)]
pub enum CodecError {
    Io(io::Error),
    TooLong,
    Invalid(serde_json::Error),
}

impl fmt::Display for CodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodecError::Io(error) => write!(formatter, "conexão: {error}"),
            CodecError::TooLong => write!(formatter, "mensagem maior que {LIMIT} bytes"),
            CodecError::Invalid(error) => write!(formatter, "mensagem inválida: {error}"),
        }
    }
}

impl std::error::Error for CodecError {}

impl From<io::Error> for CodecError {
    fn from(error: io::Error) -> Self {
        CodecError::Io(error)
    }
}

pub fn encode<T: Serialize>(message: &T) -> Result<Zeroizing<String>, CodecError> {
    let mut line = Zeroizing::new(serde_json::to_string(message).map_err(CodecError::Invalid)?);
    line.push('\n');
    Ok(line)
}

pub fn decode<T: DeserializeOwned>(line: &str) -> Result<T, CodecError> {
    serde_json::from_str(line.trim_end()).map_err(CodecError::Invalid)
}

pub fn send<T: Serialize>(writer: &mut impl Write, message: &T) -> Result<(), CodecError> {
    writer.write_all(encode(message)?.as_bytes())?;
    writer.flush()?;
    Ok(())
}

pub fn receive<T: DeserializeOwned>(reader: &mut impl BufRead) -> Result<Option<T>, CodecError> {
    let mut line = Zeroizing::new(String::new());
    let read = reader.take(LIMIT as u64 + 1).read_line(&mut line)?;
    if read == 0 {
        return Ok(None);
    }
    if read > LIMIT {
        return Err(CodecError::TooLong);
    }
    decode(&line).map(Some)
}
