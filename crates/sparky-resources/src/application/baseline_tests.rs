use sparky_hardware::stat::CpuTimes;

use super::baseline::Baseline;
use super::features::features;
use crate::domain::{Features, Metric, Reading};
use crate::telemetry::{GpuSample, MemorySample, Pressures, SystemSample, Vram};

fn sample(busy: u64, total: u64) -> SystemSample {
    SystemSample {
        cpu: Reading::Measured {
            value: CpuTimes { busy, total },
            age_ms: 0,
        },
        cores: Vec::new(),
        memory: Reading::Measured {
            value: MemorySample {
                total_kib: 1000,
                available_kib: 250,
                swap_total_kib: 0,
                swap_free_kib: 0,
            },
            age_ms: 0,
        },
        pressure: Pressures {
            cpu: Reading::Missing,
            memory: Reading::Missing,
            io: Reading::Missing,
        },
        gpu: Reading::Measured {
            value: GpuSample {
                busy_percent: Some(80),
                vram: Some(Vram { used: 2, total: 8 }),
            },
            age_ms: 0,
        },
    }
}

#[test]
fn extrai_percentuais_e_deixa_ausente_o_que_nao_foi_medido() {
    let extracted = features(
        Some(CpuTimes {
            busy: 100,
            total: 1000,
        }),
        &sample(150, 1100),
    );
    assert_eq!(extracted.get(Metric::CpuBusy), Some(50.0));
    assert_eq!(extracted.get(Metric::MemoryUsed), Some(75.0));
    assert_eq!(extracted.get(Metric::GpuBusy), Some(80.0));
    assert_eq!(extracted.get(Metric::VramUsed), Some(25.0));
    assert_eq!(extracted.get(Metric::IoPressure), None);
    assert_eq!(
        features(None, &sample(150, 1100)).get(Metric::CpuBusy),
        None
    );
}

#[test]
fn estimativas_ficam_na_faixa_e_metrica_ausente_nao_e_prevista() {
    let mut baseline = Baseline::default();
    let mut insights = Vec::new();
    for second in 0..120 {
        let value = (second as f32).min(100.0);
        let features = Features::default().with(Metric::MemoryUsed, Some(value));
        insights = baseline.observe(f64::from(second), &features);
    }
    let memory = insights
        .iter()
        .find(|insight| insight.metric == Metric::MemoryUsed);
    let estimate = memory.and_then(|insight| insight.estimate_long);
    assert!(
        estimate.is_some_and(|value| (99.0..=100.0).contains(&value)),
        "{estimate:?}"
    );
    let gpu = insights
        .iter()
        .find(|insight| insight.metric == Metric::GpuBusy);
    assert_eq!(gpu.and_then(|insight| insight.estimate_short), None);
    assert_eq!(gpu.and_then(|insight| insight.current), None);
}

#[test]
fn salto_inesperado_aparece_como_anomalia() {
    let mut baseline = Baseline::default();
    for second in 0..100 {
        baseline.observe(
            f64::from(second),
            &Features::default().with(Metric::CpuBusy, Some(10.0)),
        );
    }
    let insights = baseline.observe(
        100.0,
        &Features::default().with(Metric::CpuBusy, Some(95.0)),
    );
    let cpu = insights
        .iter()
        .find(|insight| insight.metric == Metric::CpuBusy);
    assert!(
        cpu.and_then(|insight| insight.anomaly)
            .is_some_and(|score| score > 10.0)
    );
}
