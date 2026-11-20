use super::baseline::Baseline;
use super::encoding::{OUTPUTS, Sample};
use super::reference::{base, bases};
use crate::domain::{Activities, Features, METRICS, Metric};

fn rows() -> Vec<Sample> {
    (0..40)
        .map(|index| Sample {
            at: f64::from(index) * 2.0,
            features: Features::default()
                .with(Metric::CpuBusy, Some(f64::from(index % 7) as f32 * 10.0))
                .with(Metric::MemoryUsed, Some(40.0)),
            activities: Activities::default(),
        })
        .collect()
}

#[test]
fn a_referencia_do_treino_e_a_mesma_do_servico_ao_vivo() {
    let rows = rows();
    let offline = bases(&rows);
    let mut baseline = Baseline::default();
    for (sample, expected) in rows.iter().zip(&offline) {
        let insights = baseline.observe(sample.at, &sample.features);
        assert_eq!(&base(&insights), expected);
    }
}

#[test]
fn metrica_sem_leitura_fica_sem_referencia() {
    let last = bases(&rows()).pop().unwrap_or([None; OUTPUTS]);
    assert!(last[0].is_some());
    assert_eq!(last[1], Some(40.0));
    assert_eq!(last[Metric::GpuBusy as usize], None);
    assert_eq!(last[METRICS.len() + Metric::GpuBusy as usize], None);
}
