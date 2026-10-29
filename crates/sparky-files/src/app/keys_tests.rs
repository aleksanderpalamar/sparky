use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use crate::app::keys::{Answer, Command, browse, confirm, filtering};
use crate::domain::grid::ViewMode;

fn letra(text: &str) -> Option<Command> {
    browse(&Key::Character(text.into()), Modifiers::empty())
}

fn nomeada(named: Named) -> Option<Command> {
    browse(&Key::Named(named), Modifiers::empty())
}

#[test]
fn hjkl_e_setas_navegam() {
    assert_eq!(letra("j"), Some(Command::Down));
    assert_eq!(letra("k"), Some(Command::Up));
    assert_eq!(letra("l"), Some(Command::Open));
    assert_eq!(letra("h"), Some(Command::Parent));
    assert_eq!(nomeada(Named::ArrowDown), Some(Command::Down));
    assert_eq!(nomeada(Named::Enter), Some(Command::Open));
    assert_eq!(nomeada(Named::Backspace), Some(Command::Parent));
}

#[test]
fn g_vai_ao_comeco_e_g_maiusculo_ao_fim() {
    assert_eq!(letra("g"), Some(Command::First));
    assert_eq!(letra("G"), Some(Command::Last));
    assert_eq!(nomeada(Named::End), Some(Command::Last));
}

#[test]
fn atalhos_de_arquivo_por_letra_e_por_ctrl() {
    assert_eq!(letra("y"), Some(Command::Copy));
    assert_eq!(
        browse(&Key::Character("c".into()), Modifiers::CTRL),
        Some(Command::Copy)
    );
    assert_eq!(
        browse(&Key::Character("X".into()), Modifiers::CTRL),
        Some(Command::Cut)
    );
    assert_eq!(letra("p"), Some(Command::Paste));
    assert_eq!(nomeada(Named::F2), Some(Command::Rename));
    assert_eq!(nomeada(Named::Delete), Some(Command::Trash));
    assert_eq!(letra("a"), Some(Command::Create));
}

#[test]
fn numeros_levam_aos_lugares_da_lateral() {
    assert_eq!(letra("1"), Some(Command::Place(0)));
    assert_eq!(letra("9"), Some(Command::Place(8)));
    assert_eq!(letra("0"), None);
}

#[test]
fn alt_com_seta_para_cima_sobe_e_alt_home_vai_para_casa() {
    assert_eq!(
        browse(&Key::Named(Named::ArrowUp), Modifiers::ALT),
        Some(Command::Parent)
    );
    assert_eq!(
        browse(&Key::Named(Named::Home), Modifiers::ALT),
        Some(Command::Home)
    );
}

#[test]
fn no_filtro_so_as_setas_movem() {
    assert_eq!(
        filtering(&Key::Named(Named::ArrowDown)),
        Some(Command::Down)
    );
    assert_eq!(filtering(&Key::Character("j".into())), None);
}

#[test]
fn a_confirmacao_aceita_enter_s_e_recusa_n() {
    assert_eq!(confirm(&Key::Named(Named::Enter)), Some(Answer::Yes));
    assert_eq!(confirm(&Key::Character("s".into())), Some(Answer::Yes));
    assert_eq!(confirm(&Key::Character("n".into())), Some(Answer::No));
    assert_eq!(confirm(&Key::Character("x".into())), None);
}

#[test]
fn teclas_sem_funcao_sao_ignoradas() {
    assert_eq!(letra("z"), None);
    assert_eq!(nomeada(Named::Tab), None);
    assert_eq!(browse(&Key::Character("z".into()), Modifiers::CTRL), None);
}

#[test]
fn visualizacoes_e_dispositivos_tem_atalhos_separados_dos_lugares() {
    assert_eq!(
        browse(&Key::Character("1".into()), Modifiers::CTRL),
        Some(Command::View(ViewMode::Grid))
    );
    assert_eq!(
        browse(&Key::Character("2".into()), Modifiers::CTRL),
        Some(Command::View(ViewMode::List))
    );
    assert_eq!(
        browse(&Key::Character("1".into()), Modifiers::ALT),
        Some(Command::Volume(0))
    );
    assert_eq!(
        browse(&Key::Character("u".into()), Modifiers::ALT),
        Some(Command::UnmountCurrent)
    );
    assert_eq!(letra("1"), Some(Command::Place(0)));
}

#[test]
fn atalhos_exibidos_nos_menus_de_contexto_existem() {
    assert_eq!(
        browse(
            &Key::Character("N".into()),
            Modifiers::CTRL | Modifiers::SHIFT
        ),
        Some(Command::NewFolder)
    );
    assert_eq!(browse(&Key::Character("n".into()), Modifiers::CTRL), None);
    assert_eq!(
        browse(&Key::Character("a".into()), Modifiers::CTRL),
        Some(Command::SelectAll)
    );
    assert_eq!(
        browse(&Key::Named(Named::Enter), Modifiers::ALT),
        Some(Command::Properties)
    );
}
