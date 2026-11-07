pub mod json;
pub mod plain;
pub mod view;
pub mod visual;

#[cfg(test)]
mod render_tests;

use crate::config::settings::Format;
use crate::domain::field::Field;
use crate::domain::report::Report;
use crate::domain::status::{Status, Value, hex};
use crate::i18n;
use crate::render::view::View;

pub fn render(report: &Report, view: &View) -> Result<String, String> {
    match view.format {
        Format::Json => json::render(report),
        Format::Plain => Ok(plain::render(report)),
        Format::Visual => Ok(visual::render(report, view)),
    }
}

pub fn label(field: Field) -> String {
    i18n::text(&format!("field-{}", field.key()))
}

pub fn shown(status: &Status) -> String {
    match status.value() {
        Some(Value::Text(text)) => text.clone(),
        Some(Value::Lines(lines)) => lines.join(", "),
        Some(Value::Colors(colors)) => colors
            .iter()
            .map(|color| hex(*color))
            .collect::<Vec<_>>()
            .join(" "),
        None => i18n::text("not-available"),
    }
}
