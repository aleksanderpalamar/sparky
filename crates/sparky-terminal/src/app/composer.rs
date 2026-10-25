use iced::advanced::input_method::{self, InputMethod, Preedit, Purpose};
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, renderer};
use iced::{Element, Event, Length, Rectangle, Size, window};

pub struct Composer<Message> {
    area: Rectangle,
    on_commit: fn(String) -> Message,
}

impl<Message> Composer<Message> {
    pub fn new(area: Rectangle, on_commit: fn(String) -> Message) -> Self {
        Self { area, on_commit }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Composer<Message>
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::ZERO)
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Option<Preedit>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(None::<Preedit>)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let preedit = tree.state.downcast_mut::<Option<Preedit>>();
        match event {
            Event::InputMethod(input_method::Event::Commit(text)) => {
                *preedit = None;
                shell.publish((self.on_commit)(text.clone()));
            }
            Event::InputMethod(input_method::Event::Preedit(content, selection)) => {
                *preedit = Some(Preedit {
                    content: content.clone(),
                    selection: selection.clone(),
                    text_size: None,
                });
                shell.request_redraw();
            }
            Event::InputMethod(input_method::Event::Closed) => *preedit = None,
            Event::Window(window::Event::RedrawRequested(_)) => {
                let bounds = layout.bounds();
                shell.request_input_method(&InputMethod::Enabled {
                    cursor: Rectangle {
                        x: self.area.x + bounds.x,
                        y: self.area.y + bounds.y,
                        ..self.area
                    },
                    purpose: Purpose::Terminal,
                    preedit: preedit.as_ref().map(Preedit::as_ref),
                });
            }
            _ => {}
        }
    }
}

impl<'a, Message: 'a, Theme, Renderer> From<Composer<Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(composer: Composer<Message>) -> Self {
        Element::new(composer)
    }
}
