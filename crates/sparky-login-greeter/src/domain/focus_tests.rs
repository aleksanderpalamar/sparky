use super::focus::Focus;
use super::power::{Power, key};

const ORDEM: [Focus; 7] = [
    Focus::Username,
    Focus::Password,
    Focus::Login,
    Focus::Power(Power::Shutdown),
    Focus::Power(Power::Reboot),
    Focus::Power(Power::Suspend),
    Focus::Session,
];

#[test]
fn tab_segue_a_ordem_da_tela_e_da_a_volta() {
    for (index, focus) in ORDEM.iter().enumerate() {
        assert_eq!(focus.next(), ORDEM[(index + 1) % ORDEM.len()]);
    }
}

#[test]
fn shift_tab_volta_pelo_mesmo_caminho() {
    for focus in ORDEM {
        assert_eq!(focus.next().previous(), focus);
    }
}

#[test]
fn so_os_campos_recebem_texto() {
    let campos: Vec<Focus> = ORDEM.into_iter().filter(|focus| focus.is_field()).collect();
    assert_eq!(campos, vec![Focus::Username, Focus::Password]);
}

#[test]
fn os_botoes_de_energia_seguem_a_ordem_esperada() {
    let chaves: Vec<&str> = Power::ALL.iter().map(|power| key(*power)).collect();
    assert_eq!(
        chaves,
        vec!["power-shutdown", "power-reboot", "power-suspend"]
    );
}
