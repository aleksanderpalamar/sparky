use iced::alignment::{Horizontal, Vertical};
use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::{Element, Length, Point, Rectangle, Renderer, Size, Theme};
use sparky_ipc::OutputInfo;
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::domain::diagram;
use crate::view::kit::tokens::{alpha, size};

pub const HEIGHT: f32 = 176.0;

struct Screens {
    skin: Skin,
    outputs: Vec<OutputInfo>,
}

pub fn screens<'a>(skin: Skin, outputs: &[OutputInfo]) -> Element<'a, Message> {
    Canvas::new(Screens {
        skin,
        outputs: outputs.to_vec(),
    })
    .width(Length::Fill)
    .height(Length::Fixed(HEIGHT))
    .into()
}

impl canvas::Program<Message> for Screens {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let area = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&area, self.skin.fill(alpha::NORMAL_FILL));
        frame.stroke(
            &area,
            Stroke::default()
                .with_color(self.skin.fill(alpha::HOVER_BORDER))
                .with_width(1.0),
        );
        for place in diagram::map(&self.outputs, bounds.width, bounds.height) {
            let shape = Path::rectangle(
                Point::new(place.x, place.y),
                Size::new(place.width, place.height),
            );
            let (fill, edge) = match place.focused {
                true => (
                    self.skin.accent_fill(alpha::SELECTED_FILL),
                    self.skin.accent,
                ),
                false => (
                    self.skin.fill(alpha::HOVER_FILL),
                    self.skin.fill(alpha::NORMAL_BORDER),
                ),
            };
            frame.fill(&shape, fill);
            frame.stroke(&shape, Stroke::default().with_color(edge).with_width(1.0));
            frame.fill_text(Text {
                content: place.number.to_string(),
                position: Point::new(place.x + place.width / 2.0, place.y + place.height / 2.0),
                color: self.skin.bright_foreground,
                size: size::HEADING.into(),
                font: sparky_ui::style::FONT,
                align_x: Horizontal::Center.into(),
                align_y: Vertical::Center,
                ..Text::default()
            });
        }
        vec![frame.into_geometry()]
    }
}
