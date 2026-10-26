use iced::advanced::{
    Shell,
    clipboard::Null,
    layout::{Layout, Limits},
    renderer::Headless,
    widget::Tree,
};
use iced::futures::executor::block_on;
use iced::widget::{button, text};
use iced::{Event, Length, Pixels, Point, Rectangle, Renderer, Size, mouse};

use super::drawer::drawer;
use crate::app::message::Message;

fn click(progress: f32, at: Point) -> (Vec<Message>, Size) {
    let renderer = block_on(<Renderer as Headless>::new(
        sparky_ui::style::FONT,
        Pixels(12.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let child = button(text("x"))
        .width(26.0)
        .height(Length::Fill)
        .on_press(Message::TrayActivate("app".to_owned()));
    let mut view = drawer(child.into(), 26.0, progress);
    let mut tree = Tree::new(&view);
    let size = Size::new(100.0, 26.0);
    let node = view
        .as_widget_mut()
        .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, size));
    assert_eq!(node.children()[0].size().width, 26.0);
    let mut messages = Vec::new();
    for event in [
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        view.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(event),
            Layout::new(&node),
            mouse::Cursor::Available(at),
            &renderer,
            &mut Null,
            &mut Shell::new(&mut messages),
            &Rectangle::with_size(size),
        );
    }
    (messages, node.size())
}

#[test]
fn icones_recolhidos_nao_podem_receber_cliques() {
    let (messages, size) = click(0.0, Point::new(5.0, 10.0));
    assert_eq!(size.width, 0.0);
    assert!(messages.is_empty());
}

#[test]
fn somente_a_parte_revelada_do_icone_recebe_eventos_durante_a_animacao() {
    let (messages, size) = click(0.5, Point::new(20.0, 10.0));
    assert_eq!(size.width, 13.0);
    assert!(messages.is_empty());
    let (messages, _) = click(0.5, Point::new(5.0, 10.0));
    assert!(matches!(messages.as_slice(), [Message::TrayActivate(address)] if address == "app"));
}

#[test]
fn a_gaveta_aberta_preserva_a_ativacao_do_aplicativo() {
    let (messages, size) = click(1.0, Point::new(20.0, 10.0));
    assert_eq!(size.width, 26.0);
    assert!(matches!(messages.as_slice(), [Message::TrayActivate(address)] if address == "app"));
}
