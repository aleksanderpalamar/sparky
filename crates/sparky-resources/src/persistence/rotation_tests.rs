use super::checkpoint::{Checkpoint, CheckpointError, Kind};
use super::fake_storage::FakeStorage;
use super::knowledge::{Knowledge, Restored};

const ACTIVE: &str = "sequence.ckpt";
const PREVIOUS: &str = "sequence.previous.ckpt";
const NEXT: &str = "sequence.next.ckpt";

fn model(value: f32) -> Checkpoint {
    Checkpoint {
        kind: Kind::Sequence,
        hardware: 7,
        values: vec![value; 4],
    }
}

fn stored(values: &[f32]) -> FakeStorage {
    let storage = FakeStorage::default();
    let mut knowledge = Knowledge::new(&storage);
    for value in values {
        assert!(knowledge.store(&model(*value)).is_ok());
    }
    storage
}

fn restored(storage: &FakeStorage) -> Restored {
    Knowledge::new(storage).restore(Kind::Sequence, 7)
}

#[test]
fn falha_ao_gravar_o_novo_mantem_o_atual() {
    let storage = stored(&[1.0]);
    storage.break_file(NEXT);
    assert!(Knowledge::new(&storage).store(&model(2.0)).is_err());
    storage.mend();
    assert_eq!(restored(&storage), Restored::Model(model(1.0)));
}

#[test]
fn falha_ao_mover_o_atual_mantem_o_atual() {
    let storage = stored(&[1.0, 2.0]);
    storage.break_file(PREVIOUS);
    assert!(Knowledge::new(&storage).store(&model(3.0)).is_err());
    storage.mend();
    assert_eq!(restored(&storage), Restored::Model(model(2.0)));
}

#[test]
fn queda_antes_de_promover_o_novo_usa_o_anterior() {
    let storage = stored(&[1.0]);
    let moved = storage.files.borrow_mut().remove(ACTIVE);
    if let Some(bytes) = moved {
        storage
            .files
            .borrow_mut()
            .insert(PREVIOUS.to_owned(), bytes);
    }
    assert_eq!(restored(&storage), Restored::Model(model(1.0)));
}

#[test]
fn atual_ilegivel_e_diferente_de_ausente() {
    let storage = stored(&[1.0]);
    storage.break_file(ACTIVE);
    assert_eq!(restored(&storage), Restored::Unreadable);
    let storage = stored(&[1.0, 2.0]);
    storage.break_file(ACTIVE);
    assert_eq!(restored(&storage), Restored::Model(model(1.0)));
}

#[test]
fn anterior_de_outro_hardware_nao_e_usado() {
    let storage = FakeStorage::default();
    let mut knowledge = Knowledge::new(&storage);
    let other = Checkpoint {
        hardware: 8,
        ..model(1.0)
    };
    assert!(knowledge.store(&other).is_ok());
    assert!(knowledge.store(&other).is_ok());
    storage.files.borrow_mut().remove(ACTIVE);
    assert_eq!(restored(&storage), Restored::OtherHardware);
}

#[test]
fn checkpoint_recusado_pelo_conteudo_cai_no_anterior() {
    let storage = stored(&[1.0, f32::NAN]);
    let knowledge = Knowledge::new(&storage);
    let finite = |checkpoint: &Checkpoint| checkpoint.values.iter().all(|v| v.is_finite());
    assert_eq!(
        knowledge.restore_with(Kind::Sequence, 7, finite),
        Restored::Model(model(1.0))
    );
    let storage = stored(&[f32::INFINITY]);
    assert_eq!(
        Knowledge::new(&storage).restore_with(Kind::Sequence, 7, finite),
        Restored::Rejected(CheckpointError::Values)
    );
}

#[test]
fn esquecer_tenta_todos_os_arquivos_e_pode_ser_repetido() {
    let storage = FakeStorage::default();
    let mut knowledge = Knowledge::new(&storage);
    for kind in [Kind::Sequence, Kind::Candidate, Kind::History] {
        assert!(knowledge.store(&Checkpoint { kind, ..model(1.0) }).is_ok());
    }
    storage.break_file(ACTIVE);
    assert!(knowledge.forget().is_err());
    assert_eq!(
        storage.files.borrow().keys().cloned().collect::<Vec<_>>(),
        vec![ACTIVE.to_owned()]
    );
    storage.mend();
    assert!(knowledge.forget().is_ok());
    assert!(storage.files.borrow().is_empty());
}
