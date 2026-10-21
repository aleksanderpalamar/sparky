use std::rc::Rc;

use sparky_ui::notice::Notice;

use crate::app::state::{Mode, Target};
use crate::app::test_support::{Comandos, Sparky, digitar, enter, painel_com, tecla};
use crate::domain::section::Section;
use crate::domain::system::RowId;
use crate::ports::fake::FakeCommands;

fn sistema() -> Comandos {
    Comandos(Rc::new(FakeCommands::new()
        .with("localectl status", "System Locale: LANG=pt_BR.UTF-8\nVC Keymap: br-abnt2\nX11 Layout: br\n")
        .with("localectl list-locales", "C.UTF-8\npt_BR.UTF-8\n")
        .with("localectl list-keymaps", "br-abnt2\nus\n")
        .with("localectl list-x11-keymap-layouts", "br\nus\n")
        .accepting("localectl set-locale LANG=C.UTF-8")
        .accepting("localectl set-keymap us")))
}

#[test]
fn regiao_abre_o_idioma_e_valida_a_edicao_antes_de_executar() {
    let commands = sistema();
    let (mut settings, arquivo) = painel_com(None, Sparky::default(), commands.clone());
    tecla(&mut settings, "9");
    assert_eq!(settings.section, Section::Region);
    assert_eq!(settings.count(), 3);
    enter(&mut settings);
    assert_eq!(settings.mode, Mode::Editing(Target::System(RowId::Locale)));
    assert_eq!(settings.input, "pt_BR.UTF-8");
    digitar(&mut settings, " C.UTF-8 ");
    assert!(commands.0.calls().contains(&"localectl set-locale LANG=C.UTF-8".to_owned()));
    assert_eq!(*arquivo.0.borrow(), None);
    enter(&mut settings);
    let before = commands.0.calls();
    digitar(&mut settings, "zz_INVALIDO");
    assert_eq!(commands.0.calls(), before);
    assert_eq!(settings.notice, Some(Notice::Error("valor indisponível: zz_INVALIDO".to_owned())));
}

#[test]
fn teclado_do_console_percorre_as_opcoes_e_recusa_erros_do_sistema() {
    let commands = sistema();
    let (mut settings, _) = painel_com(None, Sparky::default(), commands.clone());
    tecla(&mut settings, "9");
    tecla(&mut settings, "j");
    tecla(&mut settings, "l");
    assert!(commands.0.calls().contains(&"localectl set-keymap us".to_owned()));
    tecla(&mut settings, "j");
    tecla(&mut settings, "l");
    assert_eq!(settings.notice, Some(Notice::Error("Layout do sistema: programa não encontrado".to_owned())));
}
