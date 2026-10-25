use iced::mouse::{self, ScrollDelta};
use iced::widget::canvas::{self, Action, Event, Geometry, Program};
use iced::widget::{container, stack, text};
use iced::{Element, Length, Point, Rectangle, Renderer, Theme};

use crate::app::composer::Composer;
use crate::app::message::Message;
use crate::app::paint::paint;
use crate::app::pointer::{self, Pointer};
use crate::app::state::Terminal;
use crate::domain::tabs::TabId;

pub fn view(id: TabId, terminal: &Terminal) -> Element<'_, Message> {
    if let Some(error) = &terminal.error {
        return container(text(error))
            .padding(18)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    let painter = canvas::Canvas::new(Painter { id, terminal })
        .width(Length::Fill)
        .height(Length::Fill);
    stack![painter, Composer::new(terminal.ime_area(), Message::Commit)].into()
}

struct Painter<'a> {
    id: TabId,
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
        let lines = match mouse {
            mouse::Event::WheelScrolled { delta } => self.lines(*delta),
            _ => 0.0,
        };
        pointer::update(
            pointer,
            self.id,
            mouse,
            local.map(|at| self.spot(at)),
            cursor.is_over(bounds),
            lines,
        )
        .map(|message| Action::publish(message).and_capture())
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
