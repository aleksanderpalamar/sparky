use iced::{Point, mouse, window};

use super::contact::{Contact, contact};
use super::message::Message;
use super::test_support::{Memory, Started, dock};
use super::update::update;
use crate::domain::support::app;

#[test]
fn os_eventos_do_mouse_da_superficie_viram_contatos_do_dock() {
    let entered = iced::Event::Mouse(mouse::Event::CursorEntered);
    let left = iced::Event::Mouse(mouse::Event::CursorLeft);
    let moved = iced::Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(3.0, 4.0),
    });
    let released = iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left));
    let right = iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right));
    assert_eq!(contact(&entered), Some(Contact::Entered));
    assert_eq!(contact(&left), Some(Contact::Left));
    assert_eq!(contact(&moved), Some(Contact::Moved(Point::new(3.0, 4.0))));
    assert_eq!(contact(&released), Some(Contact::Released));
    assert_eq!(contact(&right), None);
}

#[test]
fn so_o_mouse_da_superficie_do_dock_conta() {
    let mut dock = dock(&Memory::default(), &Started::default(), Vec::new());
    let _ = update(&mut dock, Message::Catalog(vec![app("a", "A")]));
    let outra = window::Id::unique();
    let _ = update(&mut dock, Message::Surface(outra, Contact::Entered));
    assert!(!dock.hovering);
    let propria = dock.window;
    let _ = update(&mut dock, Message::Surface(propria, Contact::Entered));
    assert!(dock.hovering);
    let _ = update(&mut dock, Message::Surface(outra, Contact::Left));
    assert!(dock.hovering);
    let _ = update(&mut dock, Message::Surface(propria, Contact::Left));
    assert!(!dock.hovering);
}
