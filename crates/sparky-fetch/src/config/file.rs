use serde::Deserialize;

use crate::config::settings::{Colors, Logo, Partial};
use crate::config::values;
use crate::domain::field::Field;
use crate::i18n;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    fields: Option<Vec<String>>,
    #[serde(default)]
    hide: Vec<String>,
    format: Option<String>,
    color: Option<String>,
    logo: Option<bool>,
    layout: Option<String>,
    width: Option<usize>,
    timeout_ms: Option<u64>,
    #[serde(default)]
    colors: FileColors,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileColors {
    accent: Option<String>,
    label: Option<String>,
    value: Option<String>,
}

pub fn read(text: &str) -> Result<Partial, String> {
    let file: FileConfig = toml::from_str(text).map_err(|error| error.message().to_owned())?;
    Ok(Partial {
        fields: file.fields.as_deref().map(fields).transpose()?,
        hide: fields(&file.hide)?,
        format: choice(file.format.as_deref(), "format", values::format)?,
        color: choice(file.color.as_deref(), "color", values::color)?,
        logo: file.logo.map(logo),
        layout: choice(file.layout.as_deref(), "layout", values::layout)?,
        width: number(file.width, "width", |width| (width > 0).then_some(width))?,
        timeout: number(file.timeout_ms, "timeout_ms", values::timeout)?,
        colors: colors(&file.colors)?,
    })
}

fn fields(keys: &[String]) -> Result<Vec<Field>, String> {
    values::fields(keys.iter().map(String::as_str))
        .map_err(|key| i18n::with("error-unknown-field", "field", &key))
}

fn logo(show: bool) -> Logo {
    match show {
        true => Logo::Show,
        false => Logo::Hide,
    }
}

fn choice<T>(
    text: Option<&str>,
    key: &str,
    parse: fn(&str) -> Option<T>,
) -> Result<Option<T>, String> {
    let Some(text) = text else {
        return Ok(None);
    };
    parse(text).map(Some).ok_or_else(|| invalid(key, text))
}

fn number<N: ToString + Copy, T>(
    value: Option<N>,
    key: &str,
    parse: impl Fn(N) -> Option<T>,
) -> Result<Option<T>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    parse(value)
        .map(Some)
        .ok_or_else(|| invalid(key, &value.to_string()))
}

fn colors(file: &FileColors) -> Result<Colors, String> {
    Ok(Colors {
        accent: choice(file.accent.as_deref(), "colors.accent", values::rgb)?,
        label: choice(file.label.as_deref(), "colors.label", values::rgb)?,
        value: choice(file.value.as_deref(), "colors.value", values::rgb)?,
    })
}

fn invalid(key: &str, value: &str) -> String {
    i18n::pairs("error-invalid-value", &[("option", key), ("value", value)])
}
