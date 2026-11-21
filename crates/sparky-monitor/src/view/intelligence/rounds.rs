use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas::{self, Frame, Geometry, Path, Program, Stroke, Text};
use iced::{Color, Element, Length, Pixels, Point, Rectangle, Renderer, Theme, mouse};
use sparky_resources::application::Round;
use sparky_theme::Skin;
use sparky_ui::style::FONT;

use crate::app::message::Message;
use crate::domain::intelligence::ceiling;

const LEFT: f32 = 36.0;
const BOTTOM: f32 = 18.0;
const DIVISIONS: usize = 4;
const LABEL: f32 = 10.0;
const DOT: f32 = 2.5;
const INSET: f32 = 14.0;

pub struct Rounds {
    rounds: Vec<Round>,
    skin: Skin,
}

pub fn chart<'a>(skin: Skin, rounds: &[Round], height: f32) -> Element<'a, Message> {
    canvas::Canvas::new(Rounds {
        rounds: rounds.to_vec(),
        skin,
    })
    .width(Length::Fill)
    .height(height)
    .into()
}

impl Program<Message> for Rounds {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let plot = Rectangle {
            x: LEFT,
            y: LABEL / 2.0,
            width: (bounds.width - LEFT - LABEL).max(1.0),
            height: (bounds.height - BOTTOM - LABEL / 2.0).max(1.0),
        };
        let top = ceiling(
            self.rounds
                .iter()
                .flat_map(|round| [round.neural.unwrap_or(0.0), round.statistical]),
        );
        self.grid(&mut frame, plot, top);
        let statistical: Vec<Option<f64>> = self
            .rounds
            .iter()
            .map(|round| Some(round.statistical))
            .collect();
        let neural: Vec<Option<f64>> = self.rounds.iter().map(|round| round.neural).collect();
        line(&mut frame, plot, &statistical, top, self.skin.cyan);
        line(&mut frame, plot, &neural, top, self.skin.accent);
        vec![frame.into_geometry()]
    }
}

impl Rounds {
    fn grid(&self, frame: &mut Frame, plot: Rectangle, top: f64) {
        let stroke = Stroke::default()
            .with_color(self.skin.fill(0.12))
            .with_width(1.0);
        for step in 0..=DIVISIONS {
            let y = plot.y + plot.height * step as f32 / DIVISIONS as f32;
            frame.stroke(
                &Path::line(Point::new(plot.x, y), Point::new(plot.x + plot.width, y)),
                stroke,
            );
            let value = top * (DIVISIONS - step) as f64 / DIVISIONS as f64;
            self.label(
                frame,
                format!("{value:.1}"),
                Point::new(plot.x - 6.0, y),
                Horizontal::Right,
            );
        }
        for index in 0..self.rounds.len() {
            let x = position(plot, index, self.rounds.len());
            let point = Point::new(x, plot.y + plot.height + BOTTOM / 2.0 + 2.0);
            self.label(frame, (index + 1).to_string(), point, Horizontal::Center);
        }
    }

    fn label(&self, frame: &mut Frame, content: String, position: Point, align: Horizontal) {
        frame.fill_text(Text {
            content,
            position,
            color: self.skin.dark_foreground,
            size: Pixels(LABEL),
            font: FONT,
            align_x: align.into(),
            align_y: Vertical::Center,
            ..Text::default()
        });
    }
}

fn position(plot: Rectangle, index: usize, count: usize) -> f32 {
    let width = (plot.width - INSET * 2.0).max(1.0);
    match count {
        0 | 1 => plot.x + plot.width / 2.0,
        _ => plot.x + INSET + width * index as f32 / (count - 1) as f32,
    }
}

fn line(frame: &mut Frame, plot: Rectangle, values: &[Option<f64>], top: f64, color: Color) {
    let points: Vec<Point> = values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let value = value.filter(|value| value.is_finite())?;
            let ratio = (value / top).clamp(0.0, 1.0) as f32;
            Some(Point::new(
                position(plot, index, values.len()),
                plot.y + plot.height * (1.0 - ratio),
            ))
        })
        .collect();
    if points.len() > 1 {
        let path = Path::new(|builder| {
            builder.move_to(points[0]);
            points
                .iter()
                .skip(1)
                .for_each(|point| builder.line_to(*point));
        });
        frame.stroke(&path, Stroke::default().with_color(color).with_width(1.5));
    }
    for point in points {
        frame.fill(&Path::circle(point, DOT), color);
    }
}
