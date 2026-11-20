use super::archive::{GAP, pack_rounds, pack_samples, unpack_rounds, unpack_samples};
use super::encoding::Sample;
use super::progress::{Outcome, Round};
use crate::domain::{Activities, Activity, Features, Metric};

fn samples() -> Vec<Sample> {
    (0..3)
        .map(|index| Sample {
            at: 100.0 + f64::from(index) * 2.0,
            features: Features::default()
                .with(Metric::CpuBusy, Some(10.0 * index as f32))
                .with(Metric::GpuBusy, None),
            activities: [Activity::Gaming].into_iter().collect(),
        })
        .collect()
}

#[test]
fn a_coleta_volta_igual_e_antes_das_amostras_novas() {
    let original = samples();
    let restored = unpack_samples(&pack_samples(original.iter()), 0.0);
    assert_eq!(restored.len(), original.len());
    for (before, after) in original.iter().zip(&restored) {
        assert_eq!(after.features, before.features);
        assert_eq!(after.activities, before.activities);
        assert_eq!(after.at - before.at, -GAP - 104.0);
    }
    assert_eq!(restored.last().map(|sample| sample.at), Some(-GAP));
}

#[test]
fn coleta_corrompida_e_ignorada() {
    let mut values = pack_samples(samples().iter());
    assert!(unpack_samples(&values[1..], 0.0).is_empty());
    values[0] = f32::NAN;
    assert!(unpack_samples(&values, 0.0).is_empty());
    let mut shuffled = samples();
    shuffled.swap(0, 2);
    assert!(unpack_samples(&pack_samples(shuffled.iter()), 0.0).is_empty());
}

#[test]
fn o_contexto_sobrevive_mesmo_sem_metricas() {
    let empty = Sample {
        at: 0.0,
        features: Features::default(),
        activities: Activities::default().with(Activity::Compiling),
    };
    let restored = unpack_samples(&pack_samples([empty].iter()), 0.0);
    assert_eq!(
        restored.first().map(|sample| sample.activities),
        Some(empty.activities)
    );
}

#[test]
fn as_rodadas_voltam_com_resultado_e_erros() {
    let history = vec![
        Round {
            at_seconds: 300,
            outcome: Outcome::BehindStatistics,
            neural: Some(2.5),
            statistical: 2.25,
        },
        Round {
            at_seconds: 900,
            outcome: Outcome::NoData,
            neural: None,
            statistical: 1.5,
        },
    ];
    assert_eq!(
        unpack_rounds(&pack_rounds(13, &history)),
        Some((13, history))
    );
}

#[test]
fn rodadas_corrompidas_sao_ignoradas() {
    assert_eq!(unpack_rounds(&[]), None);
    assert_eq!(unpack_rounds(&[1.0, 2.0]), None);
    assert_eq!(unpack_rounds(&[1.0, 10.0, 9.0, 1.0, 1.0]), None);
}
