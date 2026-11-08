use std::rc::Rc;

use sparky_ui::notice::Notice;

use crate::app::state::{Mode, Target};
use crate::app::test_support::{Comandos, Sparky, digitar, enter, painel_com, salvo, tecla};
use crate::domain::section::Section;
use crate::domain::system::RowId;
use crate::ports::fake::FakeCommands;

fn sistema() -> Comandos {
    Comandos(Rc::new(
        FakeCommands::new()
            .with(
                "localectl status",
                "System Locale: LANG=pt_BR.UTF-8\nVC Keymap: br-abnt2\nX11 Layout: br\n",
            )
            .with("localectl list-locales", "C.UTF-8\npt_BR.UTF-8\n")
            .with("localectl list-keymaps", "br-abnt2\nus\n")
            .with("localectl list-x11-keymap-layouts", "br\nus\n")
            .accepting("localectl set-locale LANG=C.UTF-8")
            .accepting("localectl set-keymap us"),
    ))
}

#[test]
fn regiao_abre_o_idioma_e_valida_a_edicao_antes_de_executar() {
    let commands = sistema();
    let (mut settings, arquivo) = painel_com(None, Sparky::default(), commands.clone());
    tecla(&mut settings, "8");
    assert_eq!(settings.section, Section::Region);
    assert_eq!(settings.count(), 5);
    enter(&mut settings);
    assert_eq!(settings.mode, Mode::Editing(Target::System(RowId::Locale)));
    assert_eq!(settings.input, "pt_BR.UTF-8");
    digitar(&mut settings, " C.UTF-8 ");
    assert!(
        commands
            .0
            .calls()
            .contains(&"localectl set-locale LANG=C.UTF-8".to_owned())
    );
    assert_eq!(*arquivo.0.borrow(), None);
    enter(&mut settings);
    let before = commands.0.calls();
    digitar(&mut settings, "zz_INVALIDO");
    assert_eq!(commands.0.calls(), before);
    assert_eq!(
        settings.notice,
        Some(Notice::Error("valor indisponível: zz_INVALIDO".to_owned()))
    );
}

#[test]
fn teclado_do_console_percorre_as_opcoes_e_recusa_erros_do_sistema() {
    let commands = sistema();
    let (mut settings, _) = painel_com(None, Sparky::default(), commands.clone());
    tecla(&mut settings, "8");
    tecla(&mut settings, "j");
    tecla(&mut settings, "l");
    assert!(
        commands
            .0
            .calls()
            .contains(&"localectl set-keymap us".to_owned())
    );
    tecla(&mut settings, "j");
    tecla(&mut settings, "l");
    assert_eq!(
        settings.notice,
        Some(Notice::Error(
            "Layout do sistema: programa não encontrado".to_owned()
        ))
    );
}

fn repeticao(settings: &crate::app::state::Settings, id: RowId) -> Option<String> {
    settings
        .system_rows()
        .into_iter()
        .find(|row| row.id == id)
        .map(|row| row.value)
}

fn focar(settings: &mut crate::app::state::Settings, id: RowId) {
    let Some(index) = settings.system_rows().iter().position(|row| row.id == id) else {
        panic!("Região e idioma deveria ter a linha {id:?}");
    };
    settings.point(index);
}

#[test]
fn regiao_mostra_a_repeticao_das_teclas_da_sessao() {
    let (mut settings, _) = painel_com(
        Some("[input]\nrepeat_delay = 300\n"),
        Sparky::default(),
        sistema(),
    );
    tecla(&mut settings, "8");
    assert_eq!(settings.count(), 5);
    assert_eq!(
        repeticao(&settings, RowId::RepeatRate).as_deref(),
        Some("40")
    );
    assert_eq!(
        repeticao(&settings, RowId::RepeatDelay).as_deref(),
        Some("300")
    );
}

#[test]
fn as_setas_ajustam_a_repeticao_e_gravam_no_sparky_toml_sem_localectl() {
    let commands = sistema();
    let (mut settings, arquivo) = painel_com(None, Sparky::default(), commands.clone());
    tecla(&mut settings, "8");
    let before = commands.0.calls();
    focar(&mut settings, RowId::RepeatRate);
    tecla(&mut settings, "l");
    assert_eq!(settings.config.input.repeat_rate, 45);
    assert!(salvo(&arquivo).contains("[input]\nrepeat_rate = 45"));
    focar(&mut settings, RowId::RepeatDelay);
    tecla(&mut settings, "h");
    assert_eq!(settings.config.input.repeat_delay, 225);
    assert_eq!(commands.0.calls(), before);
}

#[test]
fn a_repeticao_digitada_e_validada_antes_de_salvar() {
    let (mut settings, arquivo) = painel_com(None, Sparky::default(), sistema());
    tecla(&mut settings, "8");
    focar(&mut settings, RowId::RepeatDelay);
    enter(&mut settings);
    assert_eq!(
        settings.mode,
        Mode::Editing(Target::System(RowId::RepeatDelay))
    );
    assert_eq!(settings.input, "250");
    digitar(&mut settings, "5000");
    assert!(matches!(settings.notice, Some(Notice::Error(_))));
    assert_eq!(salvo(&arquivo), "");
    enter(&mut settings);
    digitar(&mut settings, "400");
    assert_eq!(settings.config.input.repeat_delay, 400);
}

#[test]
fn soltar_o_controle_deslizante_grava_a_repeticao() {
    let (mut settings, arquivo) = painel_com(None, Sparky::default(), sistema());
    tecla(&mut settings, "8");
    settings.sliding = Some((Target::System(RowId::RepeatRate), 60));
    settings.release();
    assert_eq!(settings.config.input.repeat_rate, 60);
    assert!(salvo(&arquivo).contains("repeat_rate = 60"));
}
