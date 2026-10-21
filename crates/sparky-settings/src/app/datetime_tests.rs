use std::rc::Rc;

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use sparky_ui::notice::Notice;

use crate::app::message::Message;
use crate::app::state::{Mode, Target};
use crate::app::test_support::{Comandos, Sparky, digitar, enter, painel_com, tecla};
use crate::app::update::update;
use crate::domain::section::Section;
use crate::domain::system::RowId;
use crate::ports::fake::FakeCommands;

const SHOW: &str = "Timezone=America/Sao_Paulo\nTimeUSec=Wed 2026-10-21 11:35:42 -03\nCanNTP=yes\nNTP=yes\nLocalRTC=no\n";

fn sistema() -> FakeCommands {
    FakeCommands::new()
        .with("timedatectl show", SHOW)
        .with("timedatectl list-timezones", "America/Sao_Paulo\nUTC\nEurope/Lisbon\n")
}

fn comandos(fake: FakeCommands) -> Comandos {
    Comandos(Rc::new(fake))
}

#[test]
fn a_secao_mostra_quatro_linhas_e_enter_abre_o_fuso_preenchido() {
    let (mut settings, _) = painel_com(None, Sparky::default(), comandos(sistema()));
    tecla(&mut settings, "8");
    assert_eq!(settings.section, Section::DateTime);
    assert_eq!(settings.count(), 4);
    enter(&mut settings);
    assert_eq!(settings.mode, Mode::Editing(Target::System(RowId::Timezone)));
    assert_eq!(settings.input, "America/Sao_Paulo");
    tecla(&mut settings, "j");
    assert_eq!(settings.focus, 0);
}

#[test]
fn salvar_o_fuso_valido_executa_o_comando_e_rele_sem_gravar_o_toml() {
    let shared = comandos(sistema().accepting("timedatectl set-timezone Europe/Lisbon"));
    let sparky = Sparky::default();
    let (mut settings, arquivo) = painel_com(None, sparky.clone(), shared.clone());
    tecla(&mut settings, "8");
    let _ = update(&mut settings, Message::Activate(0));
    digitar(&mut settings, " Europe/Lisbon ");
    let calls = shared.0.calls();
    assert!(calls.contains(&"timedatectl set-timezone Europe/Lisbon".to_owned()));
    assert_eq!(calls.iter().filter(|call| *call == "timedatectl show").count(), 2);
    assert_eq!(settings.mode, Mode::Browse);
    assert!(settings.input.is_empty());
    assert!(matches!(&settings.notice, Some(Notice::Info(text)) if text.starts_with("Fuso horário:")));
    assert_eq!(*arquivo.0.borrow(), None);
    assert_eq!(sparky.recargas.get(), 0);
}

#[test]
fn fuso_desconhecido_mostra_erro_sem_executar_comandos() {
    let shared = comandos(sistema());
    let (mut settings, _) = painel_com(None, Sparky::default(), shared.clone());
    tecla(&mut settings, "8");
    enter(&mut settings);
    let before = shared.0.calls();
    digitar(&mut settings, " Marte/Olimpo ");
    assert_eq!(settings.notice, Some(Notice::Error("fuso desconhecido: Marte/Olimpo".to_owned())));
    assert_eq!(shared.0.calls(), before);
}

#[test]
fn cancelar_a_edicao_preserva_o_fuso() {
    let shared = comandos(sistema());
    let (mut settings, _) = painel_com(None, Sparky::default(), shared.clone());
    tecla(&mut settings, "8");
    enter(&mut settings);
    let before = shared.0.calls();
    let _ = update(&mut settings, Message::Typed("UTC".to_owned()));
    let _ = update(&mut settings, Message::Escape);
    assert_eq!(settings.mode, Mode::Browse);
    assert!(settings.input.is_empty());
    assert_eq!(shared.0.calls(), before);
}

#[test]
fn trocar_o_foco_durante_a_edicao_preserva_o_alvo_do_prompt() {
    let shared = comandos(sistema().accepting("timedatectl set-timezone UTC"));
    let (mut settings, _) = painel_com(None, Sparky::default(), shared.clone());
    tecla(&mut settings, "8");
    enter(&mut settings);
    let _ = update(&mut settings, Message::Point(3));
    digitar(&mut settings, "UTC");
    assert!(shared.0.calls().contains(&"timedatectl set-timezone UTC".to_owned()));
    assert!(!shared.0.calls().iter().any(|call| call.starts_with("timedatectl set-local-rtc")));
}

#[test]
fn enter_e_setas_alteram_ntp_e_rtc_pelo_mesmo_fluxo() {
    let shared = comandos(sistema().accepting("timedatectl set-ntp false").accepting("timedatectl set-local-rtc true"));
    let (mut settings, _) = painel_com(None, Sparky::default(), shared.clone());
    tecla(&mut settings, "8");
    tecla(&mut settings, "j");
    tecla(&mut settings, "j");
    enter(&mut settings);
    tecla(&mut settings, "j");
    tecla(&mut settings, "l");
    assert!(shared.0.calls().contains(&"timedatectl set-ntp false".to_owned()));
    assert!(shared.0.calls().contains(&"timedatectl set-local-rtc true".to_owned()));
}

#[test]
fn hora_local_e_ntp_indisponivel_nao_executam_alteracoes() {
    let shared = comandos(FakeCommands::new().with("timedatectl show", "CanNTP=no\n"));
    let (mut settings, _) = painel_com(None, Sparky::default(), shared.clone());
    tecla(&mut settings, "8");
    let before = shared.0.calls();
    for index in [1, 2] {
        let _ = update(&mut settings, Message::Activate(index));
        tecla(&mut settings, "l");
    }
    assert_eq!(settings.mode, Mode::Browse);
    assert_eq!(shared.0.calls(), before);
}

#[test]
fn recusa_do_timedatectl_vira_aviso_no_painel() {
    let (mut settings, _) = painel_com(None, Sparky::default(), comandos(sistema()));
    tecla(&mut settings, "8");
    enter(&mut settings);
    digitar(&mut settings, "UTC");
    assert_eq!(settings.notice, Some(Notice::Error("Fuso horário: programa não encontrado".to_owned())));
}

#[test]
fn f5_rele_as_informacoes_de_data_e_hora() {
    let shared = comandos(sistema());
    let (mut settings, _) = painel_com(None, Sparky::default(), shared.clone());
    tecla(&mut settings, "8");
    let before = shared.0.calls().iter().filter(|call| *call == "timedatectl show").count();
    let _ = update(&mut settings, Message::Key(Key::Named(Named::F5), Modifiers::empty()));
    let after = shared.0.calls().iter().filter(|call| *call == "timedatectl show").count();
    assert_eq!(after, before + 1);
}
