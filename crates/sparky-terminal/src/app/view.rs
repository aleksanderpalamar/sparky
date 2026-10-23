use iced::mouse::{self, Button, ScrollDelta};
use iced::widget::canvas::{self, Action, Event, Geometry, Program};
use iced::widget::stack;
use iced::{Element, Length, Point, Rectangle, Renderer, Theme};

use crate::app::composer::Composer;
use crate::app::message::Message;
use crate::app::paint::paint;
use crate::app::state::Terminal;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Pointer {
    #[default]
    Idle,
    Selecting,
}

pub fn view(terminal: &Terminal) -> Element<'_, Message> {
    let painter = canvas::Canvas::new(Painter { terminal })
        .width(Length::Fill)
        .height(Length::Fill);
    stack![painter, Composer::new(terminal.ime_area(), Message::Commit)].into()
}

struct Painter<'a> {
    terminal: &'a Terminal,
}

impl Program<Message> for Painter<'_> {
    type State = Pointer;

    fn update(
        &self,
        pointer: &mut Pointer,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let Event::Mouse(mouse) = event else {
            return None;
        };
        let local = cursor
            .position()
            .map(|position| Point::new(position.x - bounds.x, position.y - bounds.y));
        match (mouse, *pointer, local) {
            (mouse::Event::ButtonPressed(Button::Left), _, Some(at)) if cursor.is_over(bounds) => {
                *pointer = Pointer::Selecting;
                Some(Action::publish(Message::Press(self.spot(at))).and_capture())
            }
            (mouse::Event::CursorMoved { .. }, Pointer::Selecting, Some(at)) => {
                Some(Action::publish(Message::Drag(self.spot(at))))
            }
            (mouse::Event::ButtonReleased(Button::Left), Pointer::Selecting, _) => {
                *pointer = Pointer::Idle;
                None
            }
            (mouse::Event::WheelScrolled { delta }, _, _) if cursor.is_over(bounds) => {
                Some(Action::publish(Message::Wheel(self.lines(*delta))).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _pointer: &Pointer,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let terminal = self.terminal;
        let geometry = terminal.cache.draw(renderer, bounds.size(), |frame| {
            paint(frame, &terminal.screen, &terminal.palette, terminal.metrics)
        });
        vec![geometry]
    }

    fn mouse_interaction(
        &self,
        _pointer: &Pointer,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        match cursor.is_over(bounds) {
            true => mouse::Interaction::Text,
            false => mouse::Interaction::default(),
        }
    }
}

impl Painter<'_> {
    fn spot(&self, at: Point) -> crate::domain::grid::Spot {
        let size = self.terminal.size;
        self.terminal
            .metrics
            .spot(at.x, at.y, (size.columns, size.lines))
    }

    fn lines(&self, delta: ScrollDelta) -> f32 {
        match delta {
            ScrollDelta::Lines { y, .. } => y,
            ScrollDelta::Pixels { y, .. } => y / self.terminal.metrics.cell.height,
        }
    }
}
