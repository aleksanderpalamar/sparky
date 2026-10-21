use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key as CipherKey, XChaCha20Poly1305, XNonce};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::domain::index::{Entry, index_of};
use crate::domain::vault::Vault;
use crate::seal::{SealError, random};
use crate::seal::contents;
use crate::seal::kdf::{Kdf, Key};

const FORMAT: u32 = 1;
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    pub format: u32,
    pub kdf: Kdf,
    pub salt: String,
    pub index: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
struct File {
    header: Header,
    nonce: String,
    sealed: String,
}

pub struct Sealing {
    kdf: Kdf,
    salt: [u8; SALT_SIZE],
    key: Key,
}

impl fmt::Debug for Sealing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Sealing { .. }")
    }
}

impl Sealing {
    pub fn create(password: &[u8], kdf: Kdf) -> Result<Self, SealError> {
        let salt = random::<SALT_SIZE>()?;
        let key = kdf.derive(password, &salt).ok_or(SealError::Unsupported)?;
        Ok(Self { kdf, salt, key })
    }

    pub fn seal(&self, vault: &Vault) -> Result<String, SealError> {
        let header = Header { format: FORMAT, kdf: self.kdf, salt: BASE64.encode(self.salt), index: index_of(vault) };
        let nonce = random::<NONCE_SIZE>()?;
        let plain = contents::encode(vault).ok_or(SealError::Corrupt)?;
        let payload = Payload { msg: &plain, aad: &associated(&header)? };
        let sealed = cipher(&self.key).encrypt(&XNonce::from(nonce), payload).map_err(|_| SealError::Corrupt)?;
        let file = File { header, nonce: BASE64.encode(nonce), sealed: BASE64.encode(sealed) };
        serde_json::to_string_pretty(&file).map_err(|_| SealError::Corrupt)
    }
}

pub fn header(contents: &str) -> Result<Header, SealError> {
    parse(contents).map(|file| file.header)
}

pub fn open(contents: &str, password: &[u8]) -> Result<(Vault, Sealing), SealError> {
    let file = parse(contents)?;
    let kdf = file.header.kdf;
    if !kdf.plausible() {
        return Err(SealError::Corrupt);
    }
    let salt = bytes::<SALT_SIZE>(&file.header.salt)?;
    let nonce = bytes::<NONCE_SIZE>(&file.nonce)?;
    let sealed = BASE64.decode(&file.sealed).map_err(|_| SealError::Corrupt)?;
    let key = kdf.derive(password, &salt).ok_or(SealError::Corrupt)?;
    let payload = Payload { msg: &sealed, aad: &associated(&file.header)? };
    let plain = cipher(&key).decrypt(&XNonce::from(nonce), payload).map_err(|_| SealError::WrongPassword)?;
    let vault = contents::decode(&Zeroizing::new(plain)).ok_or(SealError::Corrupt)?;
    Ok((vault, Sealing { kdf, salt, key }))
}

fn parse(contents: &str) -> Result<File, SealError> {
    let file: File = serde_json::from_str(contents).map_err(|_| SealError::Corrupt)?;
    match file.header.format {
        FORMAT => Ok(file),
        _ => Err(SealError::Unsupported),
    }
}

fn associated(header: &Header) -> Result<Vec<u8>, SealError> {
    serde_json::to_vec(header).map_err(|_| SealError::Corrupt)
}

fn cipher(key: &Key) -> XChaCha20Poly1305 {
    XChaCha20Poly1305::new(CipherKey::from_slice(&key[..]))
}

fn bytes<const N: usize>(text: &str) -> Result<[u8; N], SealError> {
    let decoded = BASE64.decode(text).map_err(|_| SealError::Corrupt)?;
    decoded.try_into().map_err(|_| SealError::Corrupt)
}
