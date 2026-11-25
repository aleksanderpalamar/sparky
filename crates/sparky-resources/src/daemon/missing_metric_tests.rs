use super::brain::Brain;
use super::recall::owner;
use crate::application::Observation;
use crate::application::learning::{Plan, shape};
use crate::domain::{Activities, Features, Metric, Reading};
use crate::neural::Hybrid;
use crate::persistence::fake_storage::FakeStorage;
use crate::persistence::{Checkpoint, Kind, Knowledge};
use crate::telemetry::{Pressures, SystemSample};

const HARDWARE: u64 = 42;

fn observation(features: Features) -> Observation {
    Observation {
        activities: Activities::default(),
        features,
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

fn active(storage: &FakeStorage) -> Brain<&FakeStorage> {
    let model = Hybrid::new(shape(&Plan::default()), 3);
    let checkpoint = Checkpoint {
        kind: Kind::Sequence,
        hardware: owner(Kind::Sequence, HARDWARE),
        values: model.params.values,
    };
    assert!(Knowledge::new(storage).store(&checkpoint).is_ok());
    Brain::new(Some(Knowledge::new(storage)), HARDWARE)
}

fn neural(
    brain: &mut Brain<&FakeStorage>,
    from: u32,
    gpu: Option<f32>,
    metric: Metric,
) -> (Option<f32>, Option<f32>) {
    let features = Features::default()
        .with(Metric::CpuBusy, Some(30.0))
        .with(Metric::GpuBusy, gpu);
    let mut last = (None, None);
    for step in from..from + 40 {
        let insights = brain.insights(f64::from(step) * 2.0, &observation(features));
        if let Some(insight) = insights.iter().find(|insight| insight.metric == metric) {
            last = (insight.neural_short, insight.neural_long);
        }
    }
    last
}

#[test]
fn gpu_nunca_medida_nao_recebe_previsao_neural() {
    let storage = FakeStorage::default();
    let mut brain = active(&storage);
    assert_eq!(neural(&mut brain, 0, None, Metric::GpuBusy), (None, None));
    let cpu = neural(&mut brain, 40, None, Metric::CpuBusy);
    assert!(cpu.0.is_some() && cpu.1.is_some());
}

#[test]
fn gpu_que_volta_a_ser_medida_volta_a_ter_previsao() {
    let storage = FakeStorage::default();
    let mut brain = active(&storage);
    assert_eq!(neural(&mut brain, 0, None, Metric::GpuBusy), (None, None));
    let gpu = neural(&mut brain, 40, Some(55.0), Metric::GpuBusy);
    assert!(gpu.0.is_some() && gpu.1.is_some());
}
