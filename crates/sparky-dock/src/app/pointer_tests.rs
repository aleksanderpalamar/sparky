use iced::Point;

use super::message::Message;
use super::state::Dock;
use super::test_support::{Memory, Started, dock};
use super::update::update;
use crate::domain::support::{app, ids};

fn catalog() -> Vec<crate::domain::app::App> {
    vec![app("a", "A"), app("b", "B"), app("c", "C")]
}

fn ready(memory: &Memory, started: &Started) -> Dock {
    *memory.stored.borrow_mut() = Some(ids(&["a", "b", "c"]));
    let mut dock = dock(memory, started, Vec::new());
    let _ = update(&mut dock, Message::Catalog(catalog()));
    let _ = update(&mut dock, Message::Hover(true));
    dock
}

fn center(dock: &Dock, index: usize) -> f32 {
    let surface = dock.surface.map_or(0.0, |geometry| geometry.size.0 as f32);
    dock.layout().centers(3, surface)[index]
}

fn point(dock: &mut Dock, index: usize) {
    let x = center(dock, index);
    let _ = update(dock, Message::Pointer(Point::new(x, 40.0)));
}

#[test]
fn clicar_sem_arrastar_abre_o_aplicativo() {
    let (memory, started) = (Memory::default(), Started::default());
    let mut dock = ready(&memory, &started);
    let x = center(&dock, 1);
    let _ = update(&mut dock, Message::Pointer(Point::new(x, 40.0)));
    let _ = update(&mut dock, Message::Press("b".to_owned()));
    let _ = update(&mut dock, Message::Pointer(Point::new(x + 3.0, 40.0)));
    let _ = update(&mut dock, Message::Release);
    assert_eq!(started.0.borrow().len(), 1);
    assert_eq!(*memory.saves.borrow(), 0);
}

#[test]
fn arrastar_e_soltar_reposiciona_e_salva_sem_abrir() {
    let (memory, started) = (Memory::default(), Started::default());
    let mut dock = ready(&memory, &started);
    point(&mut dock, 0);
    let _ = update(&mut dock, Message::Press("a".to_owned()));
    point(&mut dock, 2);
    assert_eq!(super::pointer::order(&dock), ids(&["b", "c", "a"]));
    let _ = update(&mut dock, Message::Release);
    assert_eq!(dock.pinned.ids(), ids(&["b", "c", "a"]));
    assert_eq!(memory.stored.borrow().clone(), Some(ids(&["b", "c", "a"])));
    assert!(started.0.borrow().is_empty());
}

#[test]
fn sair_do_dock_durante_o_arrasto_cancela_sem_mudar_nada() {
    let (memory, started) = (Memory::default(), Started::default());
    let mut dock = ready(&memory, &started);
    point(&mut dock, 0);
    let _ = update(&mut dock, Message::Press("a".to_owned()));
    point(&mut dock, 2);
    let _ = update(&mut dock, Message::Hover(false));
    let _ = update(&mut dock, Message::Release);
    assert_eq!(dock.pinned.ids(), ids(&["a", "b", "c"]));
    assert_eq!(*memory.saves.borrow(), 0);
}

#[test]
fn o_mouse_sobre_um_icone_amplia_ele_mais_que_os_vizinhos() {
    let mut dock = ready(&Memory::default(), &Started::default());
    point(&mut dock, 1);
    let alvos: Vec<f32> = dock.zoom.iter().map(|ease| ease.target).collect();
    assert!(alvos[1] > alvos[0] && alvos[0] > 1.0);
    let _ = update(&mut dock, Message::Hover(false));
    assert!(dock.zoom.iter().all(|ease| ease.target == 1.0));
}
