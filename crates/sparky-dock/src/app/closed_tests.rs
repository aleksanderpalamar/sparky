use super::message::Message;
use super::state::OpenPicker;
use super::test_support::{Memory, Started, dock};
use super::update::update;
use crate::domain::picker::Picker;
use crate::domain::support::app;

fn shown() -> super::state::Dock {
    let mut dock = dock(
        &Memory::default(),
        &Started::default(),
        vec![app("sparky-files", "Arquivos")],
    );
    let _ = update(&mut dock, Message::Output(Some(1920)));
    let _ = update(
        &mut dock,
        Message::Catalog(vec![app("sparky-files", "Arquivos")]),
    );
    dock
}

#[test]
fn superficie_fechada_some_e_espera_um_monitor() {
    let mut dock = shown();
    let first = dock.window;
    assert!(dock.surface.is_some());
    let _ = update(&mut dock, Message::Closed(first));
    assert_eq!(dock.surface, None);
    let _ = update(&mut dock, Message::Output(None));
    assert_eq!(dock.surface, None);
    assert_eq!(dock.window, first);
}

#[test]
fn monitor_de_volta_com_a_mesma_geometria_ganha_superficie_nova() {
    let mut dock = shown();
    let (first, geometry) = (dock.window, dock.surface);
    let _ = update(&mut dock, Message::Closed(first));
    let _ = update(&mut dock, Message::Output(Some(1920)));
    assert_eq!(dock.surface, geometry);
    assert_ne!(dock.window, first);
}

#[test]
fn fechamento_desfaz_hover_menu_e_area_de_toque() {
    let mut dock = shown();
    dock.hovering = true;
    dock.menu = Some("sparky-files".to_owned());
    let window = dock.window;
    let _ = update(&mut dock, Message::Closed(window));
    assert!(!dock.hovering);
    assert_eq!(dock.menu, None);
    assert_eq!(dock.accepting, None);
}

#[test]
fn seletor_fechado_pelo_compositor_deixa_de_existir() {
    let mut dock = shown();
    let window = iced::window::Id::unique();
    dock.picker = Some(OpenPicker {
        window,
        picker: Picker::default(),
    });
    let _ = update(&mut dock, Message::Closed(window));
    assert!(dock.picker.is_none());
    assert!(dock.surface.is_some());
}

#[test]
fn janela_desconhecida_fechada_nao_mexe_na_dock() {
    let mut dock = shown();
    let (window, geometry) = (dock.window, dock.surface);
    let _ = update(&mut dock, Message::Closed(iced::window::Id::unique()));
    assert_eq!((dock.window, dock.surface), (window, geometry));
}
