use iced::alignment::Vertical;
use iced::widget::canvas::{self, Frame, Geometry, Path, Program, Stroke, Text};
use iced::{Color, Element, Length, Pixels, Point, Rectangle, Renderer, Size, Theme, mouse};
use sparky_theme::Skin;

use super::scale::{Scale, ceiling, labels};
use crate::app::message::Message;
use crate::domain::history::{History, SAMPLES};

const COLUMNS: usize = 6;
const ROWS: usize = 4;
const LABEL_SIZE: f32 = 10.0;
const LABEL_SPACE: f32 = 72.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Labels {
    Hidden,
    Shown,
}

pub struct Chart<'a> {
    series: Vec<(&'a History, Color)>,
    scale: Scale,
    labels: Labels,
    skin: Skin,
}

pub fn chart<'a>(
    skin: Skin,
    series: Vec<(&'a History, Color)>,
    scale: Scale,
    labels: Labels,
    height: f32,
) -> Element<'a, Message> {
    canvas::Canvas::new(Chart {
        series,
        scale,
        labels,
        skin,
    })
    .width(Length::Fill)
    .height(height)
    .into()
}

impl Program<Message> for Chart<'_> {
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
        let plot = match self.labels {
            Labels::Hidden => bounds.size(),
            Labels::Shown => Size::new((bounds.width - LABEL_SPACE).max(1.0), bounds.height),
        };
        frame.fill_rectangle(Point::ORIGIN, plot, self.skin.lighter_background);
        self.grid(&mut frame, plot);
        let peak = self
            .series
            .iter()
            .map(|(history, _)| history.peak())
            .fold(0.0, f32::max);
        let top = ceiling(self.scale, peak);
        for (history, color) in &self.series {
            plot_series(&mut frame, plot, history, *color, top);
        }
        if self.labels == Labels::Shown {
            self.axis(&mut frame, plot, top);
        }
        vec![frame.into_geometry()]
    }
}

impl Chart<'_> {
    fn grid(&self, frame: &mut Frame, plot: Size) {
        let line = Stroke::default()
            .with_color(self.skin.fill(0.12))
            .with_width(1.0);
        for column in 1..COLUMNS {
            let x = plot.width * column as f32 / COLUMNS as f32;
            frame.stroke(
                &Path::line(Point::new(x, 0.0), Point::new(x, plot.height)),
                line,
            );
        }
        for row in 1..ROWS {
            let y = plot.height * row as f32 / ROWS as f32;
            frame.stroke(
                &Path::line(Point::new(0.0, y), Point::new(plot.width, y)),
                line,
            );
        }
    }

    fn axis(&self, frame: &mut Frame, plot: Size, top: f32) {
        let labels = labels(self.scale, top);
        let last = (labels.len() - 1).max(1) as f32;
        for (index, label) in labels.into_iter().enumerate() {
            let y = (plot.height * index as f32 / last).clamp(0.0, plot.height - LABEL_SIZE);
            frame.fill_text(Text {
                content: label,
                position: Point::new(plot.width + 6.0, y),
                color: self.skin.dark_foreground,
                size: Pixels(LABEL_SIZE),
                font: sparky_ui::style::FONT,
                align_y: Vertical::Top,
                ..Text::default()
            });
        }
    }
}

fn plot_series(frame: &mut Frame, plot: Size, history: &History, color: Color, top: f32) {
    let values: Vec<f32> = history.values().collect();
    let offset = SAMPLES - values.len();
    let points: Vec<Point> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let x = plot.width * (offset + index) as f32 / (SAMPLES - 1) as f32;
            let y = plot.height - (value / top).clamp(0.0, 1.0) * plot.height;
            Point::new(x, y)
        })
        .collect();
    let (Some(start), Some(end)) = (points.first(), points.last()) else {
        return;
    };
    let area = Path::new(|builder| {
        builder.move_to(Point::new(start.x, plot.height));
        points.iter().for_each(|point| builder.line_to(*point));
        builder.line_to(Point::new(end.x, plot.height));
        builder.close();
    });
    frame.fill(&area, Color { a: 0.3, ..color });
    let line = Path::new(|builder| {
        builder.move_to(*start);
        points
            .iter()
            .skip(1)
            .for_each(|point| builder.line_to(*point));
    });
    frame.stroke(&line, Stroke::default().with_color(color).with_width(1.5));
}
