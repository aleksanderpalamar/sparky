use iced::{Point, Size, Task, window};

use super::{Shell, open, update};
use crate::app::message::Message;
use crate::app::test_scene::Cena;

fn casca() -> (Shell, Cena) {
    let mut cena = Cena::nova();
    let files = std::mem::replace(&mut cena.files, Cena::nova().files);
    let janela = libsparky::desk::window("sparky-files", (1000, 640), (480, 360));
    let (shell, _) = open(files, Task::none(), janela, crate::app::update::update);
    (shell, cena)
}

#[test]
fn clique_direito_abre_o_menu_como_popup_da_janela() {
    let (mut shell, _cena) = casca();
    let principal = shell.desk.main();
    let _ = update(
        &mut shell,
        Message::Moved(principal, Point::new(40.0, 60.0)),
    );
    let _ = update(&mut shell, Message::ContextBackground);
    assert!(shell.files.context.is_some());
    assert!(shell.menu.is_some());
    assert_ne!(shell.menu, Some(principal));
}

#[test]
fn ponteiro_sobre_o_menu_nao_move_o_ponteiro_da_janela() {
    let (mut shell, _cena) = casca();
    let principal = shell.desk.main();
    let _ = update(
        &mut shell,
        Message::Moved(principal, Point::new(40.0, 60.0)),
    );
    let _ = update(
        &mut shell,
        Message::Moved(window::Id::unique(), Point::new(5.0, 5.0)),
    );
    assert_eq!(shell.files.pointer, Point::new(40.0, 60.0));
}

#[test]
fn so_a_janela_principal_muda_o_tamanho_da_area() {
    let (mut shell, _cena) = casca();
    let antes = shell.files.size;
    let _ = update(
        &mut shell,
        Message::Sized(window::Id::unique(), Size::new(280.0, 300.0)),
    );
    assert_eq!(shell.files.size, antes);
    let principal = shell.desk.main();
    let _ = update(
        &mut shell,
        Message::Sized(principal, Size::new(900.0, 500.0)),
    );
    assert_eq!(shell.files.size, Size::new(900.0, 500.0));
}

#[test]
fn fechar_o_menu_por_fora_limpa_o_estado() {
    let (mut shell, _cena) = casca();
    let _ = update(&mut shell, Message::ContextBackground);
    let Some(menu) = shell.menu else {
        panic!("o menu deveria estar aberto");
    };
    let _ = update(&mut shell, Message::Closed(menu));
    assert!(shell.files.context.is_none());
    assert!(shell.menu.is_none());
}

#[test]
fn dispensar_o_menu_fecha_o_popup() {
    let (mut shell, _cena) = casca();
    let _ = update(&mut shell, Message::ContextBackground);
    assert!(shell.menu.is_some());
    let _ = update(&mut shell, Message::ContextDismiss);
    assert!(shell.menu.is_none());
}
