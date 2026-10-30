use iced::advanced::Layout;
use iced::advanced::layout::Limits;
use iced::advanced::renderer::Headless;
use iced::advanced::widget::{Id, Operation, Tree};
use iced::futures::executor::block_on;
use iced::{Element, Pixels, Rectangle, Renderer, Size};
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
