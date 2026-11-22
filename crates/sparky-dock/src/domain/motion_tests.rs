use super::motion::Ease;

#[test]
fn parado_no_alvo_ja_esta_assentado() {
    let ease = Ease::at(1.0);
    assert!(ease.settled());
}

#[test]
fn caminha_suavemente_ate_o_alvo_e_assenta() {
    let mut ease = Ease::at(0.0);
    ease.aim(1.0);
    assert!(!ease.settled());
    ease.step(0.016, 0.08);
    assert!(ease.value > 0.0 && ease.value < 0.5);
    for _ in 0..120 {
        ease.step(0.016, 0.08);
    }
    assert_eq!(ease.value, 1.0);
    assert!(ease.settled());
}

#[test]
fn um_quadro_longo_nao_passa_do_alvo() {
    let mut ease = Ease::at(0.0);
    ease.aim(1.0);
    ease.step(5.0, 0.08);
    assert_eq!(ease.value, 1.0);
}
