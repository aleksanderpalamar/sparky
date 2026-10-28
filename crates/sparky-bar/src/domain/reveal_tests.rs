use std::time::{Duration, Instant};

use super::reveal::{DURATION, Expansion, Reveal};

#[test]
fn a_gaveta_comeca_recolhida_e_termina_a_transicao_em_600_ms() {
    let mut reveal = Reveal::default();
    let now = Instant::now();
    assert_eq!(reveal.progress(), 0.0);
    assert!(!reveal.animating());
    reveal.set(Expansion::Open, now);
    reveal.advance(now + DURATION / 2);
    assert!((reveal.progress() - 0.875).abs() < 0.001);
    reveal.advance(now + DURATION);
    assert_eq!(reveal.progress(), 1.0);
    assert!(!reveal.animating());
    reveal.set(Expansion::Closed, now + DURATION);
    reveal.advance(now + DURATION * 2);
    assert_eq!(reveal.progress(), 0.0);
    assert!(!reveal.animating());
}

#[test]
fn inverter_o_movimento_nao_faz_o_conteudo_saltar() {
    let mut reveal = Reveal::default();
    let now = Instant::now();
    reveal.set(Expansion::Open, now);
    reveal.advance(now + Duration::from_millis(100));
    let before = reveal.progress();
    reveal.set(Expansion::Closed, now + Duration::from_millis(100));
    assert_eq!(reveal.progress(), before);
    reveal.advance(now + Duration::from_millis(110));
    assert!(reveal.progress() < before);
}
