use std::time::{Duration, Instant};

use super::backoff::Backoff;
use super::conductor_support::hog;

fn minutes(start: Instant, value: u64) -> Instant {
    start + Duration::from_secs(value * 60)
}

#[test]
fn sem_reversao_nenhum_processo_fica_em_pausa() {
    let backoff = Backoff::default();
    assert!(!backoff.resting("rustc", Instant::now()));
}

#[test]
fn a_primeira_reversao_pausa_o_processo_por_dez_minutos() {
    let start = Instant::now();
    let mut backoff = Backoff::default();
    backoff.failed("rustc", start);
    assert!(backoff.resting("rustc", minutes(start, 9)));
    assert!(!backoff.resting("rustc", minutes(start, 10)));
    assert!(!backoff.resting("cargo", start));
}

#[test]
fn reversoes_seguidas_dobram_a_pausa_ate_uma_hora() {
    let start = Instant::now();
    let mut backoff = Backoff::default();
    backoff.failed("rustc", start);
    backoff.failed("rustc", minutes(start, 10));
    assert!(backoff.resting("rustc", minutes(start, 29)));
    assert!(!backoff.resting("rustc", minutes(start, 30)));
    let mut at = minutes(start, 30);
    for _ in 0..6 {
        backoff.failed("rustc", at);
        at += Duration::from_secs(3600);
    }
    let last = at - Duration::from_secs(3600);
    assert!(backoff.resting("rustc", last + Duration::from_secs(3599)));
    assert!(!backoff.resting("rustc", last + Duration::from_secs(3600)));
}

#[test]
fn um_ajuste_mantido_perdoa_as_reversoes() {
    let start = Instant::now();
    let mut backoff = Backoff::default();
    backoff.failed("rustc", start);
    backoff.worked("rustc");
    assert!(!backoff.resting("rustc", start));
    backoff.failed("rustc", start);
    assert!(!backoff.resting("rustc", minutes(start, 10)));
}

#[test]
fn depois_de_muito_tempo_tranquilo_a_contagem_recomeca() {
    let start = Instant::now();
    let mut backoff = Backoff::default();
    backoff.failed("rustc", start);
    backoff.failed("rustc", minutes(start, 10));
    let later = minutes(start, 200);
    backoff.failed("rustc", later);
    assert!(!backoff.resting("rustc", later + Duration::from_secs(600)));
}

#[test]
fn a_memoria_de_reversoes_tem_limite() {
    let start = Instant::now();
    let mut backoff = Backoff::default();
    for index in 0..500 {
        backoff.failed(&format!("processo-{index}"), start);
    }
    assert!(backoff.remembered() <= 64);
    assert!(backoff.resting("processo-499", start));
}

#[test]
fn processos_em_pausa_saem_da_lista_de_candidatos() {
    let start = Instant::now();
    let mut backoff = Backoff::default();
    let consumers = hog();
    backoff.failed("rustc", start);
    assert!(backoff.eligible(&consumers, start).is_empty());
    assert_eq!(backoff.eligible(&consumers, minutes(start, 10)), consumers);
}
