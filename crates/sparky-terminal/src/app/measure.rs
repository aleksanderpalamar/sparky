use iced::advanced::graphics::text::Paragraph;
use iced::advanced::text::{self, Paragraph as _};
use iced::alignment::Vertical;
use iced::widget::text::{LineHeight, Shaping, Wrapping};
use iced::{Pixels, Size};
use sparky_ui::style::FONT;

use crate::domain::grid::Cell;

pub const FONT_SIZE: f32 = 12.0;
pub const LINE_HEIGHT: f32 = 1.3;
const SAMPLE: &str = "MMMMMMMMMMMMMMMMMMMM";

pub fn cell() -> Cell {
    let sample = text::Text {
        content: SAMPLE,
        bounds: Size::INFINITE,
        size: Pixels(FONT_SIZE),
        line_height: LineHeight::Relative(LINE_HEIGHT),
        font: FONT,
        align_x: text::Alignment::Left,
        align_y: Vertical::Top,
        shaping: Shaping::Basic,
        wrapping: Wrapping::None,
    };
    let bounds = Paragraph::with_text(sample).min_bounds();
    Cell {
        width: bounds.width / SAMPLE.len() as f32,
        height: bounds.height.ceil(),
    }
}
