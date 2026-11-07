use std::fmt;
use std::path::PathBuf;
use std::slice::Iter;

use crate::config::settings::{ColorMode, Format, Logo, Partial};
use crate::config::values;
use crate::i18n;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    Default,
    Path(PathBuf),
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Run {
        partial: Partial,
        config: ConfigSource,
    },
    Help,
    Version,
    ListFields,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    Unknown(String),
    Missing(String),
    Invalid { option: String, value: String },
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            CliError::Unknown(option) => i18n::with("error-unknown-option", "option", option),
            CliError::Missing(option) => i18n::with("error-missing-value", "option", option),
            CliError::Invalid { option, value } => i18n::pairs(
                "error-invalid-value",
                &[("option", option), ("value", value)],
            ),
        };
        formatter.write_str(&text)
    }
}

pub fn parse(arguments: &[String]) -> Result<Command, CliError> {
    let mut partial = Partial::default();
    let mut config = ConfigSource::Default;
    let mut rest = arguments.iter();
    while let Some(argument) = rest.next() {
        let (name, inline) = split(argument);
        match name {
            "-h" | "--help" => return Ok(Command::Help),
            "-V" | "--version" => return Ok(Command::Version),
            "--list-fields" => return Ok(Command::ListFields),
            "--json" => partial.format = Some(Format::Json),
            "--plain" => partial.format = Some(Format::Plain),
            "--no-color" => partial.color = Some(ColorMode::Never),
            "--logo" => partial.logo = Some(Logo::Show),
            "--no-logo" => partial.logo = Some(Logo::Hide),
            "--no-config" => config = ConfigSource::Skip,
            _ => {
                let value = value_of(name, inline, &mut rest)?;
                apply(name, &value, &mut partial, &mut config)?;
            }
        }
    }
    Ok(Command::Run { partial, config })
}

fn split(argument: &str) -> (&str, Option<&str>) {
    match argument.split_once('=') {
        Some((name, value)) if name.starts_with("--") => (name, Some(value)),
        _ => (argument, None),
    }
}

fn value_of(
    name: &str,
    inline: Option<&str>,
    rest: &mut Iter<'_, String>,
) -> Result<String, CliError> {
    if !VALUED.contains(&name) {
        return Err(CliError::Unknown(name.to_owned()));
    }
    match inline {
        Some(value) => Ok(value.to_owned()),
        None => rest
            .next()
            .cloned()
            .ok_or_else(|| CliError::Missing(name.to_owned())),
    }
}

const VALUED: [&str; 11] = [
    "--format",
    "--color",
    "--layout",
    "--fields",
    "--hide",
    "--width",
    "--timeout",
    "--accent",
    "--label-color",
    "--value-color",
    "--config",
];

fn apply(
    name: &str,
    value: &str,
    partial: &mut Partial,
    config: &mut ConfigSource,
) -> Result<(), CliError> {
    let invalid = || CliError::Invalid {
        option: name.to_owned(),
        value: value.to_owned(),
    };
    match name {
        "--format" => partial.format = Some(values::format(value).ok_or_else(invalid)?),
        "--color" => partial.color = Some(values::color(value).ok_or_else(invalid)?),
        "--layout" => partial.layout = Some(values::layout(value).ok_or_else(invalid)?),
        "--fields" => {
            partial.fields = Some(values::fields(value.split(',')).map_err(|_| invalid())?)
        }
        "--hide" => partial
            .hide
            .extend(values::fields(value.split(',')).map_err(|_| invalid())?),
        "--width" => partial.width = Some(values::width(value).ok_or_else(invalid)?),
        "--timeout" => partial.timeout = Some(milliseconds(value).ok_or_else(invalid)?),
        "--accent" => partial.colors.accent = Some(values::rgb(value).ok_or_else(invalid)?),
        "--label-color" => partial.colors.label = Some(values::rgb(value).ok_or_else(invalid)?),
        "--value-color" => partial.colors.value = Some(values::rgb(value).ok_or_else(invalid)?),
        _ => *config = ConfigSource::Path(PathBuf::from(value)),
    }
    Ok(())
}

fn milliseconds(value: &str) -> Option<std::time::Duration> {
    value.trim().parse::<u64>().ok().and_then(values::timeout)
}
