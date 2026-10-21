use std::fmt;

use sparky_config::{Config, Editor};

use crate::domain::field::{Blank, Bounds, Field, Kind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Number(i32),
    Flag(bool),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Less,
    More,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invalid {
    NotNumber,
    OutOfRange(Bounds),
    Blank,
    Toggle,
}

impl fmt::Display for Invalid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Invalid::NotNumber => write!(formatter, "digite um número inteiro"),
            Invalid::OutOfRange(bounds) => write!(formatter, "use um valor de {} a {}", bounds.min, bounds.max),
            Invalid::Blank => write!(formatter, "este campo não pode ficar vazio"),
            Invalid::Toggle => write!(formatter, "use Enter ou as setas para ligar e desligar"),
        }
    }
}

pub fn read(field: Field, config: &Config) -> Value {
    let mut copy = config.clone();
    if let Some(number) = number(field, &mut copy) {
        return Value::Number(*number);
    }
    if let Some(text) = required(field, &mut copy) {
        return Value::Text(text.clone());
    }
    if let Some(text) = optional(field, &mut copy) {
        return Value::Text(text.clone().unwrap_or_default());
    }
    Value::Flag(copy.general.default_binds)
}

pub fn store(field: Field, value: &Value, editor: &mut Editor) {
    let (table, key) = field.place();
    match value {
        Value::Number(number) => editor.set_number(table, key, *number),
        Value::Flag(flag) => editor.set_flag(table, key, *flag),
        Value::Text(text) => editor.set_text(table, key, Some(text)),
    }
}

pub fn stepped(field: Field, value: &Value, step: Step) -> Option<Value> {
    match (field.kind(), value) {
        (Kind::Number(bounds), Value::Number(current)) => {
            let delta = match step {
                Step::Less => -bounds.step,
                Step::More => bounds.step,
            };
            Some(Value::Number((current + delta).clamp(bounds.min, bounds.max)))
        }
        (Kind::Flag, Value::Flag(current)) => Some(Value::Flag(!current)),
        _ => None,
    }
}

pub fn parsed(field: Field, input: &str) -> Result<Value, Invalid> {
    let input = input.trim();
    match field.kind() {
        Kind::Number(bounds) => {
            let number = input.parse::<i32>().map_err(|_| Invalid::NotNumber)?;
            match (bounds.min..=bounds.max).contains(&number) {
                true => Ok(Value::Number(number)),
                false => Err(Invalid::OutOfRange(bounds)),
            }
        }
        Kind::Text(Blank::Refused) if input.is_empty() => Err(Invalid::Blank),
        Kind::Text(_) => Ok(Value::Text(input.to_owned())),
        Kind::Flag => Err(Invalid::Toggle),
    }
}

pub fn shown(value: &Value) -> String {
    match value {
        Value::Number(number) => number.to_string(),
        Value::Flag(true) => "ligado".to_owned(),
        Value::Flag(false) => "desligado".to_owned(),
        Value::Text(text) if text.is_empty() => "automático".to_owned(),
        Value::Text(text) => text.clone(),
    }
}

fn number(field: Field, config: &mut Config) -> Option<&mut i32> {
    match field {
        Field::GapsInner => Some(&mut config.general.gaps_inner),
        Field::GapsOuter => Some(&mut config.general.gaps_outer),
        Field::Border => Some(&mut config.general.border),
        Field::FloatWidth => Some(&mut config.general.float_width),
        Field::FloatHeight => Some(&mut config.general.float_height),
        Field::RepeatRate => Some(&mut config.input.repeat_rate),
        Field::RepeatDelay => Some(&mut config.input.repeat_delay),
        _ => None,
    }
}

fn required(field: Field, config: &mut Config) -> Option<&mut String> {
    match field {
        Field::Launcher => Some(&mut config.apps.launcher),
        Field::Files => Some(&mut config.apps.files),
        Field::Settings => Some(&mut config.apps.settings),
        Field::Screenshot => Some(&mut config.apps.screenshot),
        Field::Locker => Some(&mut config.apps.locker),
        _ => None,
    }
}

fn optional(field: Field, config: &mut Config) -> Option<&mut Option<String>> {
    match field {
        Field::Layout => Some(&mut config.input.layout),
        Field::Variant => Some(&mut config.input.variant),
        Field::Options => Some(&mut config.input.options),
        Field::Terminal => Some(&mut config.apps.terminal),
        _ => None,
    }
}
