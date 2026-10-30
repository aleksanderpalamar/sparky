use iced::mouse::{self, ScrollDelta};
use iced::widget::canvas::{self, Event, Frame, Geometry, Image, Program};
use iced::widget::image::FilterMethod;
use iced::{Point, Radians, Rectangle, Renderer, Theme};

use crate::app::keys::Command;
use crate::app::message::Message;
use crate::app::state::Ready;
use crate::domain::viewport::Area;

const PIXEL_ZOOM: u32 = 400;
const SCROLL_STEP: f32 = 60.0;

pub struct Stage<'a> {
    pub ready: &'a Ready,
}

#[derive(Debug, Default)]
pub struct Grip {
    last: Option<Point>,
    scrolled: f32,
}

impl Program<Message> for Stage<'_> {
    type State = Grip;

    fn update(
        &self,
        grip: &mut Grip,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let Event::Mouse(event) = event else {
            return None;
        };
        match event {
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                grip.last = cursor.position_over(bounds);
                None
            }
            mouse::Event::ButtonReleased(mouse::Button::Left) => {
                grip.last = None;
                None
            }
            mouse::Event::CursorMoved { position } => drag(grip, *position),
            mouse::Event::WheelScrolled { delta } if cursor.is_over(bounds) => wheel(grip, *delta),
            _ => None,
        }
    }

    fn draw(
        &self,
        _grip: &Grip,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let area = Area {
            width: bounds.width,
            height: bounds.height,
        };
        let placement = self.ready.viewport.placement(area);
        let filter = match self.ready.viewport.percent(area) >= PIXEL_ZOOM {
            true => FilterMethod::Nearest,
            false => FilterMethod::Linear,
        };
        frame.draw_image(
            Rectangle {
                x: placement.x,
                y: placement.y,
                width: placement.width,
                height: placement.height,
            },
            Image::new(self.ready.handle.clone())
                .rotation(Radians(placement.turn.radians()))
                .filter_method(filter),
        );
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        grip: &Grip,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        let area = Area {
            width: bounds.width,
            height: bounds.height,
        };
        match (
            grip.last,
            self.ready.viewport.overflows(area) && cursor.is_over(bounds),
        ) {
            (Some(_), _) => mouse::Interaction::Grabbing,
            (None, true) => mouse::Interaction::Grab,
            (None, false) => mouse::Interaction::default(),
        }
    }
}

fn drag(grip: &mut Grip, position: Point) -> Option<canvas::Action<Message>> {
    let last = grip.last?;
    grip.last = Some(position);
    Some(canvas::Action::publish(Message::Dragged(
        position.x - last.x,
        position.y - last.y,
    )))
}

fn wheel(grip: &mut Grip, delta: ScrollDelta) -> Option<canvas::Action<Message>> {
    grip.scrolled += match delta {
        ScrollDelta::Lines { y, .. } => y * SCROLL_STEP,
        ScrollDelta::Pixels { y, .. } => y,
    };
    let command = match grip.scrolled {
        scrolled if scrolled >= SCROLL_STEP => Command::ZoomIn,
        scrolled if scrolled <= -SCROLL_STEP => Command::ZoomOut,
        _ => return Some(canvas::Action::capture()),
    };
    grip.scrolled = 0.0;
    Some(canvas::Action::publish(Message::Command(command)).and_capture())
}

#[cfg(test)]
mod tests {
    use iced::Point;

    use super::{Grip, drag};

    #[test]
    fn mover_o_ponteiro_sem_botao_nao_arrasta_a_imagem() {
        let mut grip = Grip::default();
        assert!(drag(&mut grip, Point::new(10.0, 10.0)).is_none());
        assert!(drag(&mut grip, Point::new(30.0, 10.0)).is_none());
        assert_eq!(grip.last, None);
        grip.last = Some(Point::new(30.0, 10.0));
        assert!(drag(&mut grip, Point::new(40.0, 15.0)).is_some());
        assert_eq!(grip.last, Some(Point::new(40.0, 15.0)));
    }
}
