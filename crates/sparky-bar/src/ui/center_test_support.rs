use crate::app::message::Message;
use iced::{
    Element, Event, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{
        Shell,
        clipboard::Null,
        layout::{Layout, Limits},
        renderer::{Headless, Style},
        widget::Tree,
    },
    futures::executor::block_on,
    mouse,
};
use sparky_theme::Skin;

pub fn exercise(
    mut view: Element<'_, Message>,
    skin: Skin,
    size: Size,
    at: Point,
    button: mouse::Button,
) -> (Vec<u8>, Vec<Message>) {
    let mut renderer = block_on(<Renderer as Headless>::new(
        sparky_ui::style::FONT,
        Pixels(12.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let mut tree = Tree::new(&view);
    let node = view
        .as_widget_mut()
        .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, size));
    assert_eq!(node.size(), size);
    let mut messages = Vec::new();
    for event in [
        mouse::Event::ButtonPressed(button),
        mouse::Event::ButtonReleased(button),
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
    view.as_widget().draw(
        &tree,
        &mut renderer,
        &skin.theme(),
        &Style {
            text_color: skin.foreground,
        },
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &Rectangle::with_size(size),
    );
    (
        renderer.screenshot(
            Size::new(size.width as u32, size.height as u32),
            1.0,
            iced::Color::TRANSPARENT,
        ),
        messages,
    )
}

pub fn save(name: &str, pixels: &[u8], size: Size) {
    let Ok(directory) = std::env::var("SPARKY_PANEL_SNAPSHOTS") else {
        return;
    };
    let mut bytes = format!("P6\n{} {}\n255\n", size.width as u32, size.height as u32).into_bytes();
    bytes.extend(
        pixels
            .chunks_exact(4)
            .flat_map(|pixel| pixel[..3].iter().copied()),
    );
    std::fs::write(format!("{directory}/{name}.ppm"), bytes).unwrap();
}
