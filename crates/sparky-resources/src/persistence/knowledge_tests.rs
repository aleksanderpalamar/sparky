use super::checkpoint::{Checkpoint, CheckpointError, Kind};
use super::fake_storage::FakeStorage;
use super::knowledge::{Knowledge, Restored};

fn model(value: f32) -> Checkpoint {
    Checkpoint {
        kind: Kind::Sequence,
        hardware: 7,
        values: vec![value; 4],
    }
}

#[test]
fn promover_guarda_o_anterior_para_voltar_atras() {
    let storage = FakeStorage::default();
    let mut knowledge = Knowledge::new(&storage);
    assert_eq!(knowledge.restore(Kind::Sequence, 7), Restored::Missing);
    assert!(knowledge.store(&model(1.0)).is_ok());
    assert!(knowledge.store(&model(2.0)).is_ok());
    assert_eq!(
        knowledge.restore(Kind::Sequence, 7),
        Restored::Model(model(2.0))
    );
    assert!(knowledge.roll_back(Kind::Sequence).is_ok());
    assert_eq!(
        knowledge.restore(Kind::Sequence, 7),
        Restored::Model(model(1.0))
    );
    assert!(knowledge.written() > 0);
}

#[test]
fn checkpoint_corrompido_cai_no_anterior_valido() {
    let storage = FakeStorage::default();
    let mut knowledge = Knowledge::new(&storage);
    assert!(knowledge.store(&model(1.0)).is_ok());
    assert!(knowledge.store(&model(2.0)).is_ok());
    if let Some(bytes) = storage.files.borrow_mut().get_mut("sequence.ckpt") {
        bytes[20] ^= 0xFF;
    }
    assert_eq!(
        knowledge.restore(Kind::Sequence, 7),
        Restored::Model(model(1.0))
    );
}

#[test]
fn sem_anterior_valido_a_corrupcao_e_informada() {
    let storage = FakeStorage::default();
    storage
        .files
        .borrow_mut()
        .insert("sequence.ckpt".to_owned(), b"lixo".to_vec());
    let knowledge = Knowledge::new(&storage);
    assert_eq!(
        knowledge.restore(Kind::Sequence, 7),
        Restored::Rejected(CheckpointError::Truncated)
    );
}

#[test]
fn outro_hardware_invalida_a_calibracao() {
    let storage = FakeStorage::default();
    let mut knowledge = Knowledge::new(&storage);
    assert!(knowledge.store(&model(1.0)).is_ok());
    assert_eq!(
        knowledge.restore(Kind::Sequence, 8),
        Restored::OtherHardware
    );
}

#[test]
fn esquecer_apaga_todo_o_conhecimento() {
    let storage = FakeStorage::default();
    let mut knowledge = Knowledge::new(&storage);
    assert!(knowledge.store(&model(1.0)).is_ok());
    assert!(knowledge.store(&model(2.0)).is_ok());
    assert!(knowledge.forget().is_ok());
    assert!(storage.files.borrow().is_empty());
    assert!(knowledge.forget().is_ok());
}

#[test]
fn cada_tipo_tem_arquivos_proprios_e_pode_ser_descartado() {
    let storage = FakeStorage::default();
    let mut knowledge = Knowledge::new(&storage);
    let candidate = Checkpoint {
        kind: Kind::Candidate,
        ..model(3.0)
    };
    assert!(knowledge.store(&model(1.0)).is_ok());
    assert!(knowledge.store(&candidate).is_ok());
    assert_eq!(
        knowledge.restore(Kind::Candidate, 7),
        Restored::Model(candidate)
    );
    assert!(knowledge.discard(Kind::Candidate).is_ok());
    assert_eq!(knowledge.restore(Kind::Candidate, 7), Restored::Missing);
    assert!(matches!(
        knowledge.restore(Kind::Sequence, 7),
        Restored::Model(_)
    ));
}

#[test]
fn esquecer_apaga_tambem_candidato_coleta_e_rodadas() {
    let storage = FakeStorage::default();
    let mut knowledge = Knowledge::new(&storage);
    for kind in [Kind::Candidate, Kind::History, Kind::Rounds] {
        assert!(knowledge.store(&Checkpoint { kind, ..model(1.0) }).is_ok());
    }
    assert!(knowledge.forget().is_ok());
    assert!(storage.files.borrow().is_empty());
}
