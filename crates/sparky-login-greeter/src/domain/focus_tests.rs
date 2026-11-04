use super::focus::Focus;
use super::power::{Power, key};
use sparky_login_look::i18n;

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
fn os_botoes_de_energia_tem_os_nomes_da_foto_na_ordem_da_foto() {
    let nomes: Vec<String> = Power::ALL
        .iter()
        .map(|power| i18n::text(key(*power)))
        .collect();
    assert_eq!(nomes, vec!["Desligar", "Reiniciar", "Suspend"]);
}
