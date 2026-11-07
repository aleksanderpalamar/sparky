use crate::render::visual::fit;
use crate::render::visual::segment::{Line, Segment, Tone};

fn linha(partes: &[(&str, Tone)]) -> Line {
    Line::new(
        partes
            .iter()
            .map(|(texto, tom)| Segment::new(*texto, *tom))
            .collect(),
    )
}

#[test]
fn mantem_a_linha_que_cabe() {
    let original = linha(&[("CPU:", Tone::Label), (" Ryzen", Tone::Value)]);
    assert_eq!(fit::clip(original.clone(), 10), original);
}

#[test]
fn corta_com_reticencias_sem_passar_da_largura() {
    let cortada = fit::clip(
        linha(&[("CPU:", Tone::Label), (" Ryzen 7 5800X", Tone::Value)]),
        10,
    );
    assert_eq!(
        cortada,
        linha(&[("CPU:", Tone::Label), (" Ryze…", Tone::Value)])
    );
    assert_eq!(cortada.width(), 10);
}

#[test]
fn corta_no_meio_do_rotulo_quando_falta_espaco() {
    let cortada = fit::clip(
        linha(&[("Memória:", Tone::Label), (" 6 GiB", Tone::Value)]),
        5,
    );
    assert_eq!(cortada, linha(&[("Memó…", Tone::Label)]));
}

#[test]
fn respeita_caracteres_largos() {
    let cortada = fit::clip(linha(&[("日本語の名前", Tone::Value)]), 6);
    assert_eq!(cortada, linha(&[("日本…", Tone::Value)]));
    assert!(cortada.width() <= 6);
}

#[test]
fn largura_zero_nao_mostra_nada() {
    assert_eq!(fit::clip(linha(&[("x", Tone::Value)]), 0), Line::default());
}
