use super::recall::{LEGACY, Recall, stamp};
use crate::application::encoding::{CONTEXT_INPUTS, STEP_INPUTS};
use crate::application::learning::{Plan, shape};
use crate::domain::activity::ACTIVITIES;
use crate::neural::{Hybrid, Shape};
use crate::persistence::fake_storage::FakeStorage;
use crate::persistence::{Checkpoint, Kind, Knowledge};

const HARDWARE: u64 = 42;

fn legacy() -> Hybrid {
    Hybrid::new(
        Shape {
            context: ACTIVITIES.len(),
            ..shape(&Plan::default())
        },
        7,
    )
}

fn stored(storage: &FakeStorage, kind: Kind, revision: u64, model: &Hybrid) {
    let checkpoint = Checkpoint {
        kind,
        hardware: stamp(kind, HARDWARE, revision),
        values: model.params.values.clone(),
    };
    assert!(Knowledge::new(storage).store(&checkpoint).is_ok());
}

#[test]
fn o_candidato_da_revisao_anterior_e_migrado_com_a_saida_recomecando() {
    let storage = FakeStorage::default();
    let old = legacy();
    stored(&storage, Kind::Candidate, LEGACY, &old);
    let recall = Recall::new(Some(Knowledge::new(&storage)), HARDWARE);
    let plan = Plan::default();
    assert!(recall.model(Kind::Candidate, &plan).is_none());
    let Some(migrated) = recall.inherited(Kind::Candidate, &plan) else {
        panic!("deveria migrar")
    };
    let expected = old.widen(shape(&plan)).map(|mut model| {
        model.settle(1.0);
        model
    });
    assert_eq!(Some(migrated.clone()), expected);
    let kept = migrated
        .params
        .values
        .iter()
        .zip(&old.params.values)
        .take_while(|(new, old)| new == old)
        .count();
    assert!(kept > 4000, "só {kept} parâmetros da GRU preservados");
    let window = vec![vec![0.3; STEP_INPUTS]; 16];
    let context = vec![0.5; CONTEXT_INPUTS];
    assert!(
        migrated
            .predict(&window, &context)
            .iter()
            .all(|value| *value == 0.0)
    );
}

#[test]
fn modelo_de_outra_revisao_nao_e_migrado() {
    let storage = FakeStorage::default();
    stored(&storage, Kind::Candidate, 3, &legacy());
    let recall = Recall::new(Some(Knowledge::new(&storage)), HARDWARE);
    assert!(
        recall
            .inherited(Kind::Candidate, &Plan::default())
            .is_none()
    );
}
