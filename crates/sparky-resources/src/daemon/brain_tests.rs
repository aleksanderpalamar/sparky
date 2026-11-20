use super::brain::Brain;
use crate::application::encoding::WINDOW;
use crate::application::learning::{Plan, shape};
use crate::application::{Knowledge as Validity, Observation};
use crate::domain::{Activities, Features, Metric, Reading, Suspension, TrainingState};
use crate::neural::Hybrid;
use crate::persistence::fake_storage::FakeStorage;
use crate::persistence::{Checkpoint, Kind, Knowledge};
use crate::telemetry::{Pressures, SystemSample};

const HARDWARE: u64 = 42;

fn observation(value: f32) -> Observation {
    Observation {
        activities: Activities::default(),
        features: Features::default().with(Metric::CpuBusy, Some(value)),
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
    }
}

fn saved(storage: &FakeStorage, hardware: u64) {
    let model = Hybrid::new(shape(&Plan::default()), 3);
    let checkpoint = Checkpoint {
        kind: Kind::Sequence,
        hardware,
        values: model.params.values,
    };
    assert!(Knowledge::new(storage).store(&checkpoint).is_ok());
}

#[test]
fn sem_conhecimento_so_ha_estimativa_estatistica() {
    let mut brain: Brain<&FakeStorage> = Brain::new(None, HARDWARE);
    assert_eq!(brain.validity(), Validity::Unvalidated);
    let insights = brain.insights(0.0, &observation(10.0));
    assert!(
        insights
            .iter()
            .all(|insight| insight.neural_short.is_none())
    );
}

#[test]
fn modelo_salvo_na_mesma_maquina_volta_e_preve() {
    let storage = FakeStorage::default();
    saved(&storage, HARDWARE);
    let mut brain = Brain::new(Some(Knowledge::new(&storage)), HARDWARE);
    assert_eq!(brain.validity(), Validity::Validated);
    let mut insights = Vec::new();
    for step in 0..WINDOW {
        insights = brain.insights(step as f64 * 2.0, &observation(20.0));
    }
    let cpu = insights
        .iter()
        .find(|insight| insight.metric == Metric::CpuBusy);
    assert!(
        cpu.and_then(|insight| insight.neural_short)
            .is_some_and(|value| (0.0..=100.0).contains(&value))
    );
}

#[test]
fn modelo_de_outra_maquina_e_ignorado() {
    let storage = FakeStorage::default();
    saved(&storage, HARDWARE + 1);
    let brain = Brain::new(Some(Knowledge::new(&storage)), HARDWARE);
    assert_eq!(brain.validity(), Validity::Unvalidated);
}

#[test]
fn esquecer_volta_ao_estado_inicial_e_apaga_os_arquivos() {
    let storage = FakeStorage::default();
    saved(&storage, HARDWARE);
    let mut brain = Brain::new(Some(Knowledge::new(&storage)), HARDWARE);
    brain.forget();
    assert_eq!(brain.validity(), Validity::Unvalidated);
    assert!(storage.files.borrow().is_empty());
}

#[test]
fn treino_suspenso_nao_roda() {
    let mut brain: Brain<&FakeStorage> = Brain::new(None, HARDWARE);
    for step in 0..700 {
        brain.insights(f64::from(step) * 2.0, &observation(30.0));
    }
    assert!(!brain.train(TrainingState::Suspended(Suspension::SensitiveActivity), 0));
    assert!(!brain.train(TrainingState::Unavailable, 0));
    assert!(brain.train(TrainingState::Allowed, 0));
}

#[test]
fn o_progresso_comeca_sem_rodadas_e_conta_a_coleta() {
    let mut brain: Brain<&FakeStorage> = Brain::new(None, HARDWARE);
    for step in 0..50 {
        brain.insights(f64::from(step) * 2.0, &observation(30.0));
    }
    let progress = brain.progress();
    assert_eq!((progress.collected, progress.needed), (50, 600));
    assert_eq!(progress.rounds, 0);
    assert!(progress.history.is_empty());
    assert!(progress.parameters > 0);
}

#[test]
fn esquecer_durante_o_treino_descarta_a_rodada_em_andamento() {
    let storage = FakeStorage::default();
    let mut brain = Brain::new(Some(Knowledge::new(&storage)), HARDWARE);
    for step in 0..700 {
        brain.insights(f64::from(step) * 2.0, &observation(30.0));
    }
    while brain.progress().epoch == 0 {
        assert!(brain.train(TrainingState::Allowed, 0));
    }
    brain.forget();
    assert_eq!(brain.progress().epoch, 0);
    assert_eq!(brain.validity(), Validity::Unvalidated);
    assert!(storage.files.borrow().is_empty());
}

fn round_done(brain: &mut Brain<&FakeStorage>) {
    for step in 0..700 {
        brain.insights(f64::from(step) * 2.0, &observation(30.0));
    }
    while brain.progress().rounds == 0 {
        brain.train(TrainingState::Allowed, 600);
    }
}

#[test]
fn reiniciar_mantem_coleta_candidato_e_rodadas() {
    let storage = FakeStorage::default();
    let mut brain = Brain::new(Some(Knowledge::new(&storage)), HARDWARE);
    round_done(&mut brain);
    let names: Vec<String> = storage.files.borrow().keys().cloned().collect();
    for name in ["history.ckpt", "rounds.ckpt", "candidate.ckpt"] {
        assert!(
            names.iter().any(|known| known == name),
            "{name} em {names:?}"
        );
    }
    let restarted = Brain::new(Some(Knowledge::new(&storage)), HARDWARE);
    let progress = restarted.progress();
    assert!(progress.collected >= 600, "{}", progress.collected);
    assert_eq!(progress.rounds, 1);
    assert_eq!(progress.history, brain.progress().history);
}

#[test]
fn coleta_de_outra_maquina_nao_e_aproveitada() {
    let storage = FakeStorage::default();
    let mut brain = Brain::new(Some(Knowledge::new(&storage)), HARDWARE);
    round_done(&mut brain);
    let other = Brain::new(Some(Knowledge::new(&storage)), HARDWARE + 1);
    assert_eq!(other.progress().collected, 0);
    assert_eq!(other.progress().rounds, 0);
}

#[test]
fn esquecer_apaga_coleta_candidato_e_rodadas() {
    let storage = FakeStorage::default();
    let mut brain = Brain::new(Some(Knowledge::new(&storage)), HARDWARE);
    round_done(&mut brain);
    brain.forget();
    assert!(storage.files.borrow().is_empty());
    let progress = brain.progress();
    assert_eq!((progress.collected, progress.rounds), (0, 0));
    assert!(progress.history.is_empty());
}
