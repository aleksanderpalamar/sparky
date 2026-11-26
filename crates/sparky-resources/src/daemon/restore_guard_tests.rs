use super::brain::Brain;
use super::recall::{LEGACY, REVISION, Recall, stamp};
use crate::application::learning::{Plan, shape};
use crate::application::{Knowledge as Validity, Observation};
use crate::domain::activity::ACTIVITIES;
use crate::domain::{Activities, Features, Metric, Reading};
use crate::neural::{Hybrid, Shape};
use crate::persistence::fake_storage::FakeStorage;
use crate::persistence::{Checkpoint, Kind, Knowledge};
use crate::telemetry::{Pressures, SystemSample};

const HARDWARE: u64 = 42;

fn current() -> Hybrid {
    Hybrid::new(shape(&Plan::default()), 3)
}

fn legacy() -> Hybrid {
    Hybrid::new(
        Shape {
            context: ACTIVITIES.len(),
            ..shape(&Plan::default())
        },
        7,
    )
}

fn poisoned(mut model: Hybrid, value: f32) -> Hybrid {
    if let Some(first) = model.params.values.first_mut() {
        *first = value;
    }
    model
}

fn stored(storage: &FakeStorage, kind: Kind, revision: u64, model: &Hybrid) {
    let checkpoint = Checkpoint {
        kind,
        hardware: stamp(kind, HARDWARE, revision),
        values: model.params.values.clone(),
    };
    assert!(Knowledge::new(storage).store(&checkpoint).is_ok());
}

fn recall(storage: &FakeStorage) -> Recall<&FakeStorage> {
    Recall::new(Some(Knowledge::new(storage)), HARDWARE)
}

fn neural(brain: &mut Brain<&FakeStorage>) -> bool {
    let observation = Observation {
        activities: Activities::default(),
        features: Features::default().with(Metric::CpuBusy, Some(30.0)),
        sample: SystemSample {
            cpu: Reading::Missing,
            cores: Vec::new(),
            memory: Reading::Missing,
            pressure: Pressures {
                cpu: Reading::Missing,
                memory: Reading::Missing,
                io: Reading::Missing,
            },
            gpu: Reading::Missing,
        },
        processes: None,
        windows: None,
        capture: None,
    };
    (0..40)
        .flat_map(|step| brain.insights(f64::from(step) * 2.0, &observation))
        .any(|insight| insight.neural_short.is_some() || insight.neural_long.is_some())
}

#[test]
fn pesos_nao_finitos_nunca_sao_restaurados() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let storage = FakeStorage::default();
        stored(
            &storage,
            Kind::Sequence,
            REVISION,
            &poisoned(current(), value),
        );
        assert!(
            recall(&storage)
                .model(Kind::Sequence, &Plan::default())
                .is_none()
        );
    }
}

#[test]
fn com_anterior_finito_o_anterior_e_restaurado() {
    let storage = FakeStorage::default();
    stored(&storage, Kind::Sequence, REVISION, &current());
    stored(
        &storage,
        Kind::Sequence,
        REVISION,
        &poisoned(current(), f32::NAN),
    );
    assert_eq!(
        recall(&storage).model(Kind::Sequence, &Plan::default()),
        Some(current())
    );
}

#[test]
fn migrado_com_pesos_nao_finitos_nao_e_herdado() {
    let storage = FakeStorage::default();
    stored(
        &storage,
        Kind::Sequence,
        LEGACY,
        &poisoned(legacy(), f32::NAN),
    );
    assert!(
        recall(&storage)
            .inherited(Kind::Sequence, &Plan::default())
            .is_none()
    );
}

#[test]
fn checkpoint_nao_finito_deixa_so_a_estatistica() {
    let storage = FakeStorage::default();
    stored(
        &storage,
        Kind::Sequence,
        REVISION,
        &poisoned(current(), f32::NAN),
    );
    let mut brain = Brain::new(Some(Knowledge::new(&storage)), HARDWARE);
    assert_eq!(brain.validity(), Validity::Unvalidated);
    assert!(!neural(&mut brain));
}

#[test]
fn migrado_sozinho_vira_candidato_sem_previsao_publicada() {
    let storage = FakeStorage::default();
    stored(&storage, Kind::Sequence, LEGACY, &legacy());
    let mut brain = Brain::new(Some(Knowledge::new(&storage)), HARDWARE);
    assert_eq!(brain.validity(), Validity::Unvalidated);
    assert!(brain.has_candidate());
    assert!(!neural(&mut brain));
}

#[test]
fn aprovado_da_revisao_atual_restaura_normalmente() {
    let storage = FakeStorage::default();
    stored(&storage, Kind::Sequence, REVISION, &current());
    let mut brain = Brain::new(Some(Knowledge::new(&storage)), HARDWARE);
    assert_eq!(brain.validity(), Validity::Validated);
    assert!(neural(&mut brain));
}
