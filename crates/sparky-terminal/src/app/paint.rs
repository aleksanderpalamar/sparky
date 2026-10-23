use iced::advanced::graphics::geometry::Renderer;
use iced::alignment::Vertical;
use iced::widget::canvas::{Frame, Text};
use iced::widget::text::{LineHeight, Shaping};
use iced::{Color, Font, Pixels, Point, Size, font};
use sparky_theme::{Rgb, TerminalPalette};
use sparky_ui::style::FONT;

use crate::app::measure::FONT_SIZE;
use crate::domain::grid::Metrics;
use crate::domain::runs::Run;
use crate::domain::screen::{CursorSpot, Screen};
use crate::domain::style::{Decoration, Slant, Style, Weight};

const LINE: f32 = 1.0;

pub fn paint<R: Renderer>(
    frame: &mut Frame<R>,
    screen: &Screen,
    palette: &TerminalPalette,
    metrics: Metrics,
) {
    frame.fill_rectangle(Point::ORIGIN, frame.size(), color(palette.background));
    for (line, runs) in screen.rows.iter().enumerate() {
        for run in runs.iter().filter(|run| run.style.bg != palette.background) {
            frame.fill_rectangle(
                origin(metrics, line, run.column),
                span(metrics, run.cells),
                color(run.style.bg),
            );
        }
    }
    for (line, runs) in screen.rows.iter().enumerate() {
        for run in runs {
            letters(frame, metrics, line, run);
        }
    }
    if let Some(cursor) = screen.cursor {
        block(frame, metrics, cursor, palette);
    }
}

fn letters<R: Renderer>(frame: &mut Frame<R>, metrics: Metrics, line: usize, run: &Run) {
    let place = origin(metrics, line, run.column);
    if !run.text.trim().is_empty() {
        frame.fill_text(text(
            &run.text,
            place,
            run.style.fg,
            font_of(&run.style),
            metrics,
            shaping(&run.text),
        ));
    }
    let width = span(metrics, run.cells).width;
    match run.style.decoration {
        Decoration::Underline => frame.fill_rectangle(
            Point::new(place.x, place.y + metrics.cell.height - 2.0 * LINE),
            Size::new(width, LINE),
            color(run.style.fg),
        ),
        Decoration::Strikeout => frame.fill_rectangle(
            Point::new(place.x, place.y + metrics.cell.height / 2.0),
            Size::new(width, LINE),
            color(run.style.fg),
        ),
        Decoration::None => {}
    }
}

fn block<R: Renderer>(
    frame: &mut Frame<R>,
    metrics: Metrics,
    cursor: CursorSpot,
    palette: &TerminalPalette,
) {
    let place = origin(metrics, cursor.line, cursor.column);
    frame.fill_rectangle(place, span(metrics, 1), color(palette.cursor));
    let character = cursor.character.to_string();
    if !character.trim().is_empty() {
        frame.fill_text(text(
            &character,
            place,
            palette.cursor_text,
            FONT,
            metrics,
            shaping(&character),
        ));
    }
}

fn text(
    content: &str,
    position: Point,
    foreground: Rgb,
    font: Font,
    metrics: Metrics,
    shaping: Shaping,
) -> Text {
    Text {
        content: content.to_owned(),
        position,
        color: color(foreground),
        size: Pixels(FONT_SIZE),
        line_height: LineHeight::Absolute(Pixels(metrics.cell.height)),
        font,
        align_y: Vertical::Top,
        shaping,
        ..Text::default()
    }
}

fn shaping(content: &str) -> Shaping {
    match content.is_ascii() {
        true => Shaping::Basic,
        false => Shaping::Advanced,
    }
}

fn font_of(style: &Style) -> Font {
    let weight = match style.weight {
        Weight::Bold => font::Weight::Bold,
        Weight::Normal => font::Weight::Normal,
    };
    let slant = match style.slant {
        Slant::Italic => font::Style::Italic,
        Slant::Upright => font::Style::Normal,
    };
    Font {
        weight,
        style: slant,
        ..FONT
    }
}

fn origin(metrics: Metrics, line: usize, column: usize) -> Point {
    Point::new(
        metrics.padding + column as f32 * metrics.cell.width,
        metrics.padding + line as f32 * metrics.cell.height,
    )
}

fn span(metrics: Metrics, cells: usize) -> Size {
    Size::new(cells as f32 * metrics.cell.width, metrics.cell.height)
}

fn color(rgb: Rgb) -> Color {
    Color::from_rgb8(rgb.red, rgb.green, rgb.blue)
}
