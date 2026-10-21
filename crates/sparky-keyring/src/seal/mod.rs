pub mod contents;
pub mod file;
pub mod kdf;

#[cfg(test)]
mod seal_tests;

use std::fmt;

pub use file::{Header, Sealing, header, open};
pub use kdf::Kdf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealError {
    Corrupt,
    Unsupported,
    WrongPassword,
    Random,
}

impl fmt::Display for SealError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            SealError::Corrupt => "o arquivo do cofre está corrompido",
            SealError::Unsupported => "o arquivo do cofre está num formato desconhecido",
            SealError::WrongPassword => "senha incorreta",
            SealError::Random => "o sistema não forneceu números aleatórios",
        };
        formatter.write_str(text)
    }
}

impl std::error::Error for SealError {}

pub fn random<const N: usize>() -> Result<[u8; N], SealError> {
    let mut buffer = [0u8; N];
    getrandom::fill(&mut buffer).map_err(|_| SealError::Random)?;
    Ok(buffer)
}
