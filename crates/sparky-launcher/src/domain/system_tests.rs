use sparky_ipc::SessionAction;

use crate::domain::app::Runs;
use crate::domain::search::{Order, rank};
use crate::domain::system::entries;

fn nomes(busca: &str) -> Vec<String> {
    rank(&entries(), busca, Order::AsListed).into_iter().map(|entrada| entrada.name.clone()).collect()
}

#[test]
fn as_opcoes_seguem_a_ordem_do_menu_do_hyprland() {
    assert_eq!(nomes(""), ["Bloquear", "Suspender", "Sair da sessão", "Reiniciar", "Desligar"]);
}

#[test]
fn cada_opcao_pede_uma_acao_de_sessao() {
    let acoes: Vec<Runs> = entries().into_iter().map(|entrada| entrada.runs).collect();
    let esperadas = [SessionAction::Lock, SessionAction::Suspend, SessionAction::Logout, SessionAction::Reboot, SessionAction::Poweroff];
    assert_eq!(acoes, esperadas.map(Runs::Session));
}

#[test]
fn digitar_o_comeco_acha_a_opcao() {
    assert_eq!(nomes("desl").first().map(String::as_str), Some("Desligar"));
    assert_eq!(nomes("d").first().map(String::as_str), Some("Desligar"));
    assert_eq!(nomes("reboot").first().map(String::as_str), Some("Reiniciar"));
    assert_eq!(nomes("sair").first().map(String::as_str), Some("Sair da sessão"));
}
