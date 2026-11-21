use super::checkpoint::{Checkpoint, CheckpointError, Kind, decode, encode};

fn sample() -> Checkpoint {
    Checkpoint {
        kind: Kind::Sequence,
        hardware: 0xABCD,
        values: vec![0.5, -1.25, 3.0],
    }
}

#[test]
fn o_checkpoint_vai_e_volta_intacto() {
    assert_eq!(decode(&encode(&sample())), Ok(sample()));
}

#[test]
fn qualquer_byte_alterado_e_detectado() {
    let original = encode(&sample());
    for index in 7..original.len() {
        let mut damaged = original.clone();
        damaged[index] ^= 0x10;
        assert!(decode(&damaged).is_err(), "byte {index}");
    }
}

#[test]
fn arquivo_cortado_ou_estranho_e_recusado() {
    let bytes = encode(&sample());
    assert_eq!(
        decode(&bytes[..bytes.len() - 3]),
        Err(CheckpointError::Checksum)
    );
    assert_eq!(decode(&bytes[..10]), Err(CheckpointError::Truncated));
    assert_eq!(
        decode(b"nada disso aqui e um modelo"),
        Err(CheckpointError::Magic)
    );
    let mut future = bytes.clone();
    future[4] = 9;
    assert_eq!(decode(&future), Err(CheckpointError::Format));
}

#[test]
fn arquivo_grande_demais_nem_e_lido() {
    assert_eq!(
        decode(&vec![0; super::checkpoint::LIMIT + 1]),
        Err(CheckpointError::TooLarge)
    );
}
