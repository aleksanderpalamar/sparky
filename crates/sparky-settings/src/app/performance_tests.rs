use sparky_config::ResourceMode;
use sparky_resources::domain::{Mode, Switch};
use sparky_resources::protocol::Request;

use super::message::Message;
use super::state::Settings;
use super::test_support::{Arquivo, Gerenciador, Sparky, painel};
use super::update::update;
use crate::domain::performance::Knob;
use crate::domain::section::Section;

fn aberto(gerenciador: &Gerenciador) -> (Settings, Arquivo) {
    let (mut settings, arquivo) = painel(Some(""), Sparky::default());
    settings.services.resources = Box::new(gerenciador.clone());
    let _ = update(&mut settings, Message::Section(Section::Performance));
    (settings, arquivo)
}

fn salvo(arquivo: &Arquivo) -> String {
    arquivo.0.borrow().clone().unwrap_or_default()
}

#[test]
fn abrir_a_secao_mostra_o_estado_do_servico() {
    let (settings, _) = aberto(&Gerenciador::default());
    assert!(settings.performance.status.is_some());
    assert_eq!(settings.performance.error, None);
}

#[test]
fn desligar_grava_na_configuracao_e_avisa_o_servico() {
    let gerenciador = Gerenciador::default();
    let (mut settings, arquivo) = aberto(&gerenciador);
    let _ = update(&mut settings, Message::PerformanceToggle(Knob::Enabled));
    assert!(salvo(&arquivo).contains("enabled = false"));
    assert!(!settings.config.resources.enabled);
    assert_eq!(
        gerenciador.pedidos.borrow().last(),
        Some(&Request::SetEnabled {
            switch: Switch::Off
        })
    );
}

#[test]
fn trocar_o_modo_grava_e_aplica_na_hora() {
    let gerenciador = Gerenciador::default();
    let (mut settings, arquivo) = aberto(&gerenciador);
    let _ = update(
        &mut settings,
        Message::PerformanceMode(ResourceMode::Conservative),
    );
    assert!(salvo(&arquivo).contains("mode = \"conservative\""));
    assert_eq!(
        gerenciador.pedidos.borrow().last(),
        Some(&Request::SetMode {
            mode: Mode::Conservative
        })
    );
}

#[test]
fn apagar_o_conhecimento_pede_confirmacao() {
    let gerenciador = Gerenciador::default();
    let (mut settings, _) = aberto(&gerenciador);
    let _ = update(&mut settings, Message::PerformanceForget(false));
    assert!(settings.performance.confirming);
    assert!(gerenciador.pedidos.borrow().is_empty());
    let _ = update(&mut settings, Message::PerformanceCancel);
    assert!(!settings.performance.confirming);
    let _ = update(&mut settings, Message::PerformanceForget(false));
    let _ = update(&mut settings, Message::PerformanceForget(true));
    assert_eq!(
        gerenciador.pedidos.borrow().as_slice(),
        [Request::ForgetKnowledge]
    );
}

#[test]
fn servico_fora_do_ar_aparece_como_erro_e_a_configuracao_ainda_e_gravada() {
    let gerenciador = Gerenciador {
        fora_do_ar: true,
        ..Gerenciador::default()
    };
    let (mut settings, arquivo) = aberto(&gerenciador);
    assert!(settings.performance.status.is_none());
    assert!(settings.performance.error.is_some());
    let _ = update(&mut settings, Message::PerformanceToggle(Knob::Learning));
    assert!(salvo(&arquivo).contains("learning = false"));
}
