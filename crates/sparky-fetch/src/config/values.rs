use std::time::Duration;

use sparky_theme::Rgb;

use crate::config::settings::{ColorMode, Format, Layout};
use crate::domain::field::Field;

pub fn format(text: &str) -> Option<Format> {
    match text.trim() {
        "visual" => Some(Format::Visual),
        "json" => Some(Format::Json),
        "plain" => Some(Format::Plain),
        _ => None,
    }
}

pub fn color(text: &str) -> Option<ColorMode> {
    match text.trim() {
        "auto" => Some(ColorMode::Auto),
        "always" => Some(ColorMode::Always),
        "never" => Some(ColorMode::Never),
        _ => None,
    }
}

pub fn layout(text: &str) -> Option<Layout> {
    match text.trim() {
        "classic" => Some(Layout::Classic),
        "stacked" => Some(Layout::Stacked),
        _ => None,
    }
}

pub fn fields<'a>(keys: impl IntoIterator<Item = &'a str>) -> Result<Vec<Field>, String> {
    keys.into_iter()
        .filter(|key| !key.trim().is_empty())
        .map(|key| Field::named(key).ok_or_else(|| key.trim().to_owned()))
        .collect()
}

pub fn width(text: &str) -> Option<usize> {
    text.trim().parse::<usize>().ok().filter(|width| *width > 0)
}

pub fn timeout(milliseconds: u64) -> Option<Duration> {
    (milliseconds > 0).then(|| Duration::from_millis(milliseconds))
}

pub fn rgb(text: &str) -> Option<Rgb> {
    Rgb::parse(text)
}
