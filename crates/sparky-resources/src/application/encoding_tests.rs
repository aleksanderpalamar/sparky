use super::encoding::{
    OUTPUTS, STEP_INPUTS, Sample, WINDOW, absolute, decode, example_at, examples, step_input,
};
use crate::domain::{Activities, Activity, Features, Metric};

fn sample(at: f64, features: Features) -> Sample {
    Sample {
        at,
        features,
        activities: Activities::default(),
    }
}

fn rows(count: usize) -> Vec<Sample> {
    (0..count)
        .map(|index| {
            sample(
                index as f64 * 2.0,
                Features::default().with(Metric::CpuBusy, Some(index as f32)),
            )
        })
        .collect()
}

#[test]
fn cada_passo_leva_valores_mascaras_intervalo_e_contexto() {
    let input = step_input(
        &Sample {
            activities: [Activity::Idle, Activity::Gaming].into_iter().collect(),
            ..sample(
                0.0,
                Features::default().with(Metric::MemoryUsed, Some(50.0)),
            )
        },
        2.0,
    );
    assert_eq!(input.len(), STEP_INPUTS);
    assert_eq!(input[1], 0.5);
    assert_eq!(input[8], 1.0);
    assert_eq!(input[0], 0.0);
    assert_eq!(input[7], 0.0);
    assert_eq!(input[14], 0.2);
    let context = &input[15..];
    assert_eq!(context.iter().sum::<f32>(), 2.0);
    assert_eq!(context[0], 1.0);
    assert_eq!(context[4], 1.0);
}

#[test]
fn os_alvos_sao_a_variacao_em_relacao_ao_valor_atual() {
    let found = examples(&rows(60));
    let Some((window, targets)) = found.first() else {
        panic!("deveria haver exemplos");
    };
    assert_eq!(window.len(), WINDOW);
    assert_eq!(targets.len(), OUTPUTS);
    assert_eq!(targets[0], Some(5.0 / 100.0));
    assert_eq!(targets[7], Some(30.0 / 100.0));
    assert_eq!(targets[1], None);
}

#[test]
fn sem_valor_atual_nao_ha_alvo() {
    let mut rows = rows(60);
    rows[WINDOW - 1].features = Features::default();
    assert!(example_at(&rows, WINDOW).is_none());
    assert!(example_at(&rows, WINDOW + 1).is_some());
}

#[test]
fn sem_futuro_suficiente_nao_ha_exemplo() {
    assert!(examples(&rows(WINDOW)).is_empty());
}

#[test]
fn a_saida_soma_a_variacao_ao_valor_atual_na_faixa() {
    let current = Features::default()
        .with(Metric::CpuBusy, Some(40.0))
        .with(Metric::MemoryUsed, Some(95.0));
    let decoded = decode(&[0.1, 0.1, -0.2], &current);
    assert_eq!(decoded, vec![50.0, 100.0, 0.0]);
}

#[test]
fn saida_zero_repete_o_valor_atual() {
    let current = Features::default().with(Metric::CpuBusy, Some(37.0));
    assert_eq!(decode(&[0.0; OUTPUTS], &current)[0], 37.0);
}

#[test]
fn o_alvo_absoluto_volta_a_partir_da_variacao() {
    let current = Features::default().with(Metric::CpuBusy, Some(40.0));
    assert_eq!(
        absolute(&[Some(0.25), Some(0.1)], &current),
        vec![Some(65.0), None]
    );
}
