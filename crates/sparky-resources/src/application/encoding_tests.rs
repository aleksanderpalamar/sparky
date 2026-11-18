use super::encoding::{OUTPUTS, STEP_INPUTS, WINDOW, decode, examples, step_input};
use crate::domain::{Features, Metric};

fn rows(count: usize) -> Vec<(f64, Features)> {
    (0..count)
        .map(|index| {
            (
                index as f64 * 2.0,
                Features::default().with(Metric::CpuBusy, Some(index as f32)),
            )
        })
        .collect()
}

#[test]
fn cada_passo_leva_valores_mascaras_e_intervalo() {
    let input = step_input(
        &Features::default().with(Metric::MemoryUsed, Some(50.0)),
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
fn exemplos_usam_janela_fixa_e_alvos_do_futuro() {
    let found = examples(&rows(60));
    let Some((window, targets)) = found.first() else {
        panic!("deveria haver exemplos");
    };
    assert_eq!(window.len(), WINDOW);
    assert_eq!(targets.len(), OUTPUTS);
    assert_eq!(targets[0], Some((15.0 + 5.0) / 100.0));
    assert_eq!(targets[7], Some((15.0 + 30.0) / 100.0));
    assert_eq!(targets[1], None);
}

#[test]
fn sem_futuro_suficiente_nao_ha_exemplo() {
    assert!(examples(&rows(WINDOW)).is_empty());
}

#[test]
fn a_saida_volta_para_percentual_na_faixa() {
    assert_eq!(decode(&[0.5, 1.5, -0.1]), vec![50.0, 100.0, 0.0]);
}
