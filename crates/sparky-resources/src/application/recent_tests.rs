use super::encoding::{OUTPUTS, Sample, WINDOW};
use super::recent::Recent;
use super::reference::{Base, misses};
use crate::domain::{Activities, Features, Metric};

fn sample(at: f64, cpu: f32) -> Sample {
    Sample {
        at,
        features: Features::default().with(Metric::CpuBusy, Some(cpu)),
        activities: Activities::default(),
    }
}

fn expecting(cpu: f32) -> Base {
    let mut base = [None; OUTPUTS];
    base[0] = Some(cpu);
    base
}

#[test]
fn guarda_90_segundos_mas_nunca_menos_que_a_janela() {
    let mut recent = Recent::default();
    for step in 0..200 {
        recent.push(sample(f64::from(step) * 2.0, 10.0), expecting(10.0));
    }
    assert_eq!(recent.len(), 46);
    assert!(recent.window().is_some_and(|window| window.len() == WINDOW));
    let mut slow = Recent::default();
    for step in 0..40 {
        slow.push(sample(f64::from(step) * 30.0, 10.0), expecting(10.0));
    }
    assert_eq!(slow.len(), WINDOW);
}

#[test]
fn sem_amostras_suficientes_nao_ha_janela() {
    let mut recent = Recent::default();
    recent.push(sample(0.0, 10.0), expecting(10.0));
    assert!(recent.window().is_none());
    assert!(!recent.is_empty());
}

#[test]
fn o_erro_recente_compara_a_previsao_antiga_com_o_valor_de_agora() {
    let mut recent = Recent::default();
    let mut samples = Vec::new();
    let mut bases = Vec::new();
    for step in 0..10 {
        let at = f64::from(step) * 2.0;
        let cpu = if step < 9 { 20.0 } else { 50.0 };
        recent.push(sample(at, cpu), expecting(20.0));
        samples.push(sample(at, cpu));
        bases.push(expecting(20.0));
    }
    let missed = recent.misses();
    assert_eq!(missed[0], Some(0.3));
    assert_eq!(missed, misses(&samples, &bases));
    assert_eq!(missed[1], None);
}

#[test]
fn sem_previsao_antiga_no_horizonte_nao_ha_erro() {
    let samples = [sample(0.0, 10.0), sample(100.0, 30.0)];
    let bases = [expecting(10.0), expecting(10.0)];
    assert_eq!(misses(&samples, &bases)[0], None);
}
