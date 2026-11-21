use super::encoding::{
    CONTEXT_INPUTS, OUTPUTS, STEP_INPUTS, Sample, WINDOW, absolute, context, decode, example_at,
    examples, step_input,
};
use super::reference::{Base, bases, misses};
use crate::domain::{Activities, Activity, Features, METRICS, Metric};

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

fn base_with(cpu_short: f32) -> Base {
    let mut base = [None; OUTPUTS];
    base[0] = Some(cpu_short);
    base
}

#[test]
fn cada_passo_leva_valores_mascaras_e_intervalo() {
    let input = step_input(
        &sample(
            0.0,
            Features::default().with(Metric::MemoryUsed, Some(50.0)),
        ),
        2.0,
    );
    assert_eq!(input.len(), STEP_INPUTS);
    assert_eq!(input[1], 0.5);
    assert_eq!(input[8], 1.0);
    assert_eq!(input[0], 0.0);
    assert_eq!(input[7], 0.0);
    assert_eq!(input[STEP_INPUTS - 1], 0.2);
}

#[test]
fn o_contexto_leva_atividades_previsao_e_erro_recente_da_estatistica() {
    let current = Features::default().with(Metric::CpuBusy, Some(20.0));
    let mut missed = [None; OUTPUTS];
    missed[2] = Some(-0.25);
    let flags = context(
        [Activity::Idle, Activity::Gaming].into_iter().collect(),
        &base_with(35.0),
        &current,
        &missed,
    );
    assert_eq!(flags.len(), CONTEXT_INPUTS);
    let (activities, rest) = flags.split_at(CONTEXT_INPUTS - OUTPUTS * 2);
    let (expected, errors) = rest.split_at(OUTPUTS);
    assert_eq!(activities.iter().sum::<f32>(), 2.0);
    assert_eq!((activities[0], activities[4]), (1.0, 1.0));
    assert_eq!(expected[0], 0.15);
    assert!(expected[1..].iter().all(|value| *value == 0.0));
    assert_eq!(errors[2], -0.25);
    assert_eq!(errors.iter().filter(|value| **value != 0.0).count(), 1);
}

#[test]
fn o_alvo_e_o_erro_da_estatistica() {
    let rows = rows(60);
    let bases = bases(&rows);
    let Some(example) = example_at(&rows, &bases, WINDOW) else {
        panic!("deveria haver exemplo");
    };
    assert_eq!(example.window.len(), WINDOW);
    assert_eq!(example.targets.len(), OUTPUTS);
    let reference = bases[WINDOW - 1];
    let later = (WINDOW - 1 + 5) as f32;
    assert_eq!(
        example.targets[0],
        reference[0].map(|value| (later - value) / 100.0)
    );
    assert_eq!(example.targets[1], None);
    assert_eq!(absolute(&example.targets, &reference)[0], Some(later));
}

#[test]
fn sem_referencia_nao_ha_alvo() {
    let rows = rows(60);
    let empty = vec![[None; OUTPUTS]; rows.len()];
    assert!(example_at(&rows, &empty, WINDOW).is_none());
}

#[test]
fn sem_futuro_suficiente_nao_ha_exemplo() {
    assert!(examples(&rows(WINDOW)).is_empty());
}

#[test]
fn a_saida_corrige_a_previsao_da_estatistica() {
    let current = Features::default()
        .with(Metric::CpuBusy, Some(40.0))
        .with(Metric::MemoryUsed, Some(95.0));
    let mut base = base_with(45.0);
    base[1] = Some(96.0);
    let decoded = decode(&[0.1, 0.1, -0.2], &base, &current);
    assert_eq!(decoded, vec![55.0, 100.0, 0.0]);
}

#[test]
fn saida_zero_repete_a_estatistica_ou_o_valor_atual() {
    let current = Features::default()
        .with(Metric::CpuBusy, Some(37.0))
        .with(Metric::MemoryUsed, Some(20.0));
    let decoded = decode(&[0.0; OUTPUTS], &base_with(41.0), &current);
    assert_eq!((decoded[0], decoded[1]), (41.0, 20.0));
    assert_eq!(decoded.len(), OUTPUTS);
    assert_eq!(METRICS.len() * 2, OUTPUTS);
}

#[test]
fn o_exemplo_leva_o_contexto_do_instante_atual() {
    let mut rows = rows(60);
    rows[WINDOW - 1].activities = [Activity::Compiling].into_iter().collect();
    let bases = bases(&rows);
    let found = example_at(&rows, &bases, WINDOW).map(|example| example.context);
    let expected = context(
        [Activity::Compiling].into_iter().collect(),
        &bases[WINDOW - 1],
        &rows[WINDOW - 1].features,
        &misses(&rows[..WINDOW], &bases[..WINDOW]),
    );
    assert_eq!(found, Some(expected));
}
