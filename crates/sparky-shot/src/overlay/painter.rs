use iced::mouse::{self, Button};
use iced::widget::canvas::{self, Action, Event, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};
use sparky_theme::Skin;
use sparky_tiling::Rect;

use crate::overlay::Message;

const DIM: f32 = 0.45;
const BORDER: f32 = 2.0;

pub struct Painter {
    pub selection: Rect,
    pub skin: Skin,
}

impl canvas::Program<Message> for Painter {
    type State = ();

    fn update(&self, _state: &mut (), event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> Option<Action<Message>> {
        let point = cursor.position_in(bounds)?;
        let (x, y) = (point.x.round() as i32, point.y.round() as i32);
        let message = match event {
            Event::Mouse(mouse::Event::ButtonPressed(Button::Left)) => Message::Pressed(x, y),
            Event::Mouse(mouse::Event::ButtonReleased(Button::Left)) => Message::Released(x, y),
            Event::Mouse(mouse::Event::CursorMoved { .. }) => Message::Moved(x, y),
            _ => return None,
        };
        Some(Action::publish(message))
    }

    fn draw(&self, _state: &(), renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let dim = Color { a: DIM, ..Color::BLACK };
        for shade in shades(self.selection, bounds.size()) {
            frame.fill_rectangle(shade.position(), shade.size(), dim);
        }
        let area = rectangle(self.selection);
        frame.stroke(
            &Path::rectangle(area.position(), area.size()),
            Stroke::default().with_color(self.skin.accent).with_width(BORDER),
        );
        frame.fill_text(Text {
            content: format!("{} × {}", self.selection.width, self.selection.height),
            position: Point::new(area.x + 6.0, area.y + 6.0),
            color: self.skin.bright_foreground,
            size: 14.0.into(),
            ..Text::default()
        });
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(&self, _state: &(), _bounds: Rectangle, _cursor: mouse::Cursor) -> mouse::Interaction {
        mouse::Interaction::Crosshair
    }
}

fn rectangle(rect: Rect) -> Rectangle {
    Rectangle::new(
        Point::new(rect.x as f32, rect.y as f32),
        Size::new(rect.width as f32, rect.height as f32),
    )
}

fn shades(selection: Rect, size: Size) -> [Rectangle; 4] {
    let area = rectangle(selection);
    [
        Rectangle::new(Point::ORIGIN, Size::new(size.width, area.y)),
        Rectangle::new(Point::new(0.0, area.y + area.height), Size::new(size.width, (size.height - area.y - area.height).max(0.0))),
        Rectangle::new(Point::new(0.0, area.y), Size::new(area.x, area.height)),
        Rectangle::new(Point::new(area.x + area.width, area.y), Size::new((size.width - area.x - area.width).max(0.0), area.height)),
    ]
}
