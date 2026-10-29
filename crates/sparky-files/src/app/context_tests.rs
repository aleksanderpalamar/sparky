use iced::Point;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

use super::message::Message;
use super::state::Mode;
use super::test_scene::Cena;
use crate::domain::context::{Action, Target};

#[test]
fn clique_direito_seleciona_o_item_e_abre_o_menu_no_ponteiro() {
    let mut scene = Cena::nova();
    scene.mandar(Message::Pointer(Point::new(330.0, 180.0)));
    scene.mandar(Message::ContextEntry(1));
    let menu = scene.files.context.as_ref().unwrap();
    assert_eq!(menu.target, Target::File);
    assert_eq!(menu.name.as_deref(), Some("a.txt"));
    assert_eq!(menu.point, Point::new(330.0, 180.0));
    assert_eq!(scene.selecionado().as_deref(), Some("a.txt"));
}

#[test]
fn clique_na_area_vazia_e_escape_nao_mudam_a_selecao() {
    let mut scene = Cena::nova();
    let selected = scene.selecionado();
    scene.mandar(Message::ContextBackground);
    assert_eq!(
        scene.files.context.as_ref().map(|menu| menu.target),
        Some(Target::Background)
    );
    assert_eq!(scene.selecionado(), selected);
    scene.mandar(Message::Escape);
    assert!(scene.files.context.is_none());
    assert_eq!(scene.selecionado(), selected);
}

#[test]
fn clique_direito_preserva_multisselecao_quando_item_esta_marcado() {
    let mut scene = Cena::nova();
    scene.files.listing.select(0);
    scene.files.listing.toggle_mark();
    scene.files.listing.select(1);
    scene.files.listing.toggle_mark();
    scene.mandar(Message::ContextEntry(1));
    assert_eq!(scene.files.listing.marked_count(), 2);
    scene.mandar(Message::ContextEntry(0));
    assert_eq!(scene.files.listing.marked_count(), 2);
}

#[test]
fn propriedades_do_item_e_da_area_vazia_mostram_o_alvo_correto() {
    let mut scene = Cena::nova();
    scene.mandar(Message::ContextEntry(1));
    scene.mandar(Message::ContextAction(Action::Properties));
    let details = scene.files.details.as_ref().unwrap();
    assert_eq!(details.path, scene.work().join("a.txt"));
    assert_eq!(details.size, 2);
    scene.mandar(Message::Escape);
    assert!(scene.files.details.is_none());
    scene.mandar(Message::ContextBackground);
    scene.mandar(Message::ContextAction(Action::Properties));
    assert_eq!(scene.files.details.as_ref().unwrap().path, scene.work());
}

#[test]
fn novas_pasta_e_documento_preservam_o_tipo_ao_editar_o_nome() {
    let mut scene = Cena::nova();
    scene.mandar(Message::ContextBackground);
    scene.mandar(Message::ContextAction(Action::NewFolder));
    scene.mandar(Message::Typed("Pasta nova".to_owned()));
    scene.mandar(Message::Submit);
    assert!(scene.work().join("Pasta nova").is_dir());
    scene.mandar(Message::ContextBackground);
    scene.mandar(Message::ContextAction(Action::NewFile));
    scene.mandar(Message::Typed("notas.txt".to_owned()));
    scene.mandar(Message::Submit);
    assert!(scene.work().join("notas.txt").is_file());
}

#[test]
fn atalhos_do_teclado_executam_as_acoes_dos_menus() {
    let mut scene = Cena::nova();
    scene.mandar(Message::Key(Key::Character("a".into()), Modifiers::CTRL));
    assert_eq!(
        scene.files.listing.marked_count(),
        scene.files.listing.len()
    );
    scene.mandar(Message::Escape);
    scene.files.listing.select(1);
    scene.mandar(Message::Key(Key::Named(Named::Enter), Modifiers::ALT));
    assert_eq!(
        scene.files.details.as_ref().unwrap().path,
        scene.work().join("a.txt")
    );
    scene.mandar(Message::Escape);
    scene.mandar(Message::Key(
        Key::Character("N".into()),
        Modifiers::CTRL | Modifiers::SHIFT,
    ));
    scene.mandar(Message::Submit);
    assert!(scene.work().join("Nova pasta").is_dir());
}

#[test]
fn teclas_nao_agem_por_tras_de_menu_ou_dialogo_aberto() {
    let mut scene = Cena::nova();
    scene.mandar(Message::ContextEntry(1));
    scene.mandar(Message::Key(Key::Named(Named::Delete), Modifiers::empty()));
    assert_eq!(scene.files.mode, Mode::Browse);
    scene.mandar(Message::ContextAction(Action::Properties));
    scene.mandar(Message::Key(Key::Character("d".into()), Modifiers::empty()));
    assert_eq!(scene.files.mode, Mode::Browse);
    assert!(scene.files.details.is_some());
}
