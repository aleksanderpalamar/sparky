use iced::advanced::Layout;
use iced::advanced::layout::Limits;
use iced::advanced::renderer::{Headless, Style};
use iced::advanced::widget::{Id, Operation, Tree};
use iced::futures::executor::block_on;
use iced::{Element, Pixels, Rectangle, Renderer, Size, mouse};
use sparky_theme::Skin;
use sparky_ui::style::FONT;

use super::message::Message;

#[derive(Default)]
struct Texts(Vec<String>);

impl Operation for Texts {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn text(&mut self, _: Option<&Id>, _: Rectangle, text: &str) {
        self.0.push(text.to_owned());
    }
}

pub fn visible_texts(mut view: Element<'_, Message>) -> Vec<String> {
    let renderer = block_on(<Renderer as Headless>::new(
        FONT,
        Pixels(12.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let mut tree = Tree::new(&view);
    let node = view.as_widget_mut().layout(
        &mut tree,
        &renderer,
        &Limits::new(Size::ZERO, Size::new(700.0, 500.0)),
    );
    let mut texts = Texts::default();
    view.as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut texts);
    texts.0
}

pub fn snapshot(mut view: Element<'_, Message>, skin: Skin, size: Size, name: &str) {
    let Ok(folder) = std::env::var("SPARKY_SETTINGS_SNAPSHOTS") else {
        return;
    };
    let Some(mut renderer) = block_on(<Renderer as Headless>::new(
        FONT,
        Pixels(12.0),
        Some("tiny-skia"),
    )) else {
        return;
    };
    let mut tree = Tree::new(&view);
    let node = view
        .as_widget_mut()
        .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, size));
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
    let (width, height) = (size.width as u32, size.height as u32);
    let pixels = renderer.screenshot(Size::new(width, height), 1.0, skin.background);
    let mut bytes = format!("P6\n{width} {height}\n255\n").into_bytes();
    bytes.extend(pixels.chunks_exact(4).flat_map(|pixel| pixel[..3].to_vec()));
    let _ = std::fs::write(format!("{folder}/{name}.ppm"), bytes);
}
