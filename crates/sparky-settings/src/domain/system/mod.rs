pub mod power;
pub mod profiles;

#[cfg(test)]
mod power_tests;

use crate::domain::value::Step;
use crate::ports::commands::{CommandError, CommandRunner};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Control {
    Choice,
    Level,
    Toggle(bool),
    Reading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowId {
    PowerProfile,
    Brightness,
    Battery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: RowId,
    pub label: &'static str,
    pub value: String,
    pub control: Control,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Step(Step),
    Activate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    SetProfile(String),
    SetBrightness { device: String, percent: u8 },
}

impl Request {
    pub fn command(&self) -> (&'static str, Vec<String>) {
        match self {
            Request::SetProfile(profile) => ("powerprofilesctl", vec!["set".to_owned(), profile.clone()]),
            Request::SetBrightness { device, percent } => {
                ("brightnessctl", vec!["-d".to_owned(), device.clone(), "set".to_owned(), format!("{percent}%")])
            }
        }
    }
}

pub fn perform(commands: &dyn CommandRunner, request: &Request) -> Result<(), CommandError> {
    let (program, arguments) = request.command();
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    commands.run(program, &arguments).map(drop)
}
