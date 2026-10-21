use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use sparky_ui::notice::Notice;

use crate::app::message::Message;
use crate::app::state::{Mode, Source};
use crate::app::test_support::{Sparky, digitar, enter, painel, salvo, tecla};
use crate::app::update::update;
use crate::domain::section::Section;

#[test]
fn sem_arquivo_ajustar_cria_o_sparky_toml_e_pede_a_recarga() {
    let sparky = Sparky::default();
    let (mut settings, arquivo) = painel(None, sparky.clone());
    tecla(&mut settings, "l");
    assert!(salvo(&arquivo).contains("gaps_inner = 6"));
    assert_eq!(settings.config.general.gaps_inner, 6);
    assert_eq!(sparky.recargas.get(), 1);
    assert_eq!(settings.notice, Some(Notice::Info("Folga entre janelas: 6 (salvo e aplicado)".to_owned())));
}

#[test]
fn mudar_um_valor_guarda_os_comentarios_do_usuario() {
    let (mut settings, arquivo) = painel(Some("# meu sparky\n[general]\ngaps_inner = 5\n"), Sparky::default());
    tecla(&mut settings, "h");
    assert_eq!(salvo(&arquivo), "# meu sparky\n[general]\ngaps_inner = 4\n");
}

#[test]
fn numero_digitado_e_validado_antes_de_salvar() {
    let (mut settings, arquivo) = painel(None, Sparky::default());
    tecla(&mut settings, "j");
    enter(&mut settings);
    assert_eq!(settings.input, "10");
    digitar(&mut settings, "abc");
    assert!(matches!(settings.notice, Some(Notice::Error(_))));
    assert_eq!(salvo(&arquivo), "");
    enter(&mut settings);
    digitar(&mut settings, "20");
    assert_eq!(settings.config.general.gaps_outer, 20);
}

#[test]
fn layout_que_o_xkb_nao_conhece_e_recusado() {
    let (mut settings, arquivo) = painel(None, Sparky::default());
    tecla(&mut settings, "2");
    assert_eq!(settings.section, Section::Keyboard);
    enter(&mut settings);
    digitar(&mut settings, "xx");
    assert_eq!(settings.notice, Some(Notice::Error("layout desconhecido: xx".to_owned())));
    assert_eq!(salvo(&arquivo), "");
    enter(&mut settings);
    digitar(&mut settings, "br");
    assert_eq!(settings.config.input.layout.as_deref(), Some("br"));
}

#[test]
fn enter_liga_e_desliga_os_atalhos_padrao() {
    let (mut settings, _) = painel(None, Sparky::default());
    for _ in 0..5 {
        tecla(&mut settings, "j");
    }
    enter(&mut settings);
    assert!(!settings.config.general.default_binds);
    assert_eq!(settings.mode, Mode::Browse);
}

#[test]
fn inicializacao_adiciona_edita_e_remove_comandos() {
    let (mut settings, _) = painel(None, Sparky::default());
    tecla(&mut settings, "4");
    enter(&mut settings);
    digitar(&mut settings, "mako");
    assert_eq!(settings.config.autostart, ["mako"]);
    enter(&mut settings);
    assert_eq!(settings.input, "mako");
    digitar(&mut settings, "mako -c ~/.mako");
    assert_eq!(settings.config.autostart, ["mako -c ~/.mako"]);
    tecla(&mut settings, "d");
    assert!(settings.config.autostart.is_empty());
}

#[test]
fn esc_desiste_da_edicao_sem_salvar() {
    let (mut settings, arquivo) = painel(None, Sparky::default());
    enter(&mut settings);
    let _ = update(&mut settings, Message::Typed("40".to_owned()));
    let _ = update(&mut settings, Message::Escape);
    assert_eq!(settings.mode, Mode::Browse);
    assert_eq!(salvo(&arquivo), "");
}

#[test]
fn arquivo_quebrado_nao_e_sobrescrito() {
    let (mut settings, arquivo) = painel(Some("[general\nborder = "), Sparky::default());
    assert!(matches!(settings.source, Source::Broken(_)));
    tecla(&mut settings, "l");
    assert_eq!(salvo(&arquivo), "[general\nborder = ");
    assert!(matches!(settings.notice, Some(Notice::Error(_))));
}

#[test]
fn com_o_sparky_fora_do_ar_salva_e_avisa_que_vale_depois() {
    let sparky = Sparky { fora_do_ar: true, ..Sparky::default() };
    let (mut settings, arquivo) = painel(None, sparky);
    tecla(&mut settings, "l");
    assert!(salvo(&arquivo).contains("gaps_inner = 6"));
    assert_eq!(
        settings.notice,
        Some(Notice::Info("Folga entre janelas: 6 (salvo; vale quando o sparky iniciar)".to_owned()))
    );
}

#[test]
fn tab_troca_de_secao_e_os_atalhos_listam_os_padroes() {
    let (mut settings, _) = painel(None, Sparky::default());
    let _ = update(&mut settings, Message::Key(Key::Named(Named::Tab), Modifiers::SHIFT));
    assert_eq!(settings.section, Section::About);
    tecla(&mut settings, "5");
    assert_eq!(settings.section, Section::Shortcuts);
    assert!(settings.count() > 20);
    tecla(&mut settings, "j");
    assert_eq!(settings.focus, 1);
}
