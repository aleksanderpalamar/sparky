const MAGIC: &[u8; 4] = b"SPKR";
pub const FORMAT: u16 = 1;
pub const LIMIT: usize = 1 << 20;
const HEADER: usize = 4 + 2 + 1 + 8 + 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Sequence = 1,
    Context = 2,
    Candidate = 3,
    History = 4,
    Rounds = 5,
}

impl Kind {
    pub const ALL: [Kind; 5] = [
        Kind::Sequence,
        Kind::Context,
        Kind::Candidate,
        Kind::History,
        Kind::Rounds,
    ];
}

#[derive(Debug, Clone, PartialEq)]
pub struct Checkpoint {
    pub kind: Kind,
    pub hardware: u64,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointError {
    TooLarge,
    Truncated,
    Magic,
    Format,
    Kind,
    Checksum,
}

pub fn encode(checkpoint: &Checkpoint) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(HEADER + checkpoint.values.len() * 4 + 8);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&FORMAT.to_le_bytes());
    bytes.push(checkpoint.kind as u8);
    bytes.extend_from_slice(&checkpoint.hardware.to_le_bytes());
    bytes.extend_from_slice(&(checkpoint.values.len() as u32).to_le_bytes());
    for value in &checkpoint.values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    let sum = checksum(&bytes);
    bytes.extend_from_slice(&sum.to_le_bytes());
    bytes
}

pub fn decode(bytes: &[u8]) -> Result<Checkpoint, CheckpointError> {
    if bytes.len() > LIMIT {
        return Err(CheckpointError::TooLarge);
    }
    let body = bytes
        .len()
        .checked_sub(8)
        .filter(|body| *body >= HEADER)
        .ok_or(CheckpointError::Truncated)?;
    if &bytes[..4] != MAGIC {
        return Err(CheckpointError::Magic);
    }
    if u16::from_le_bytes([bytes[4], bytes[5]]) != FORMAT {
        return Err(CheckpointError::Format);
    }
    if checksum(&bytes[..body]) != u64::from_le_bytes(array(&bytes[body..])) {
        return Err(CheckpointError::Checksum);
    }
    let kind = Kind::ALL
        .into_iter()
        .find(|kind| *kind as u8 == bytes[6])
        .ok_or(CheckpointError::Kind)?;
    let count = u32::from_le_bytes(array(&bytes[15..19])) as usize;
    if HEADER + count * 4 != body {
        return Err(CheckpointError::Truncated);
    }
    let values = bytes[HEADER..body]
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes(array(chunk)))
        .collect();
    Ok(Checkpoint {
        kind,
        hardware: u64::from_le_bytes(array(&bytes[7..15])),
        values,
    })
}

pub fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn array<const N: usize>(bytes: &[u8]) -> [u8; N] {
    let mut out = [0; N];
    out.copy_from_slice(&bytes[..N]);
    out
}
