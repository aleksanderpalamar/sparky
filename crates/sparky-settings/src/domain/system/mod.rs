pub mod datetime;
pub mod key_value;
pub mod power;
pub mod profiles;

#[cfg(test)]
mod datetime_tests;
#[cfg(test)]
mod power_tests;

use crate::domain::value::Step;
use crate::ports::commands::{CommandError, CommandRunner};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Control {
    Choice,
    Level,
    Text,
    Toggle(bool),
    Reading,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowId {
    PowerProfile,
    Brightness,
    Battery,
    Timezone,
    LocalTime,
    NetworkTime,
    LocalClock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: RowId,
    pub label: &'static str,
    pub value: String,
    pub control: Control,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Step(Step),
    Activate,
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    SetProfile(String),
    SetBrightness { device: String, percent: u8 },
    SetTimezone(String),
    SetNetworkTime(bool),
    SetLocalClock(bool),
}

impl Request {
    pub fn command(&self) -> (&'static str, Vec<String>) {
        match self {
            Request::SetProfile(profile) => ("powerprofilesctl", vec!["set".to_owned(), profile.clone()]),
            Request::SetBrightness { device, percent } => {
                ("brightnessctl", vec!["-d".to_owned(), device.clone(), "set".to_owned(), format!("{percent}%")])
            }
            Request::SetTimezone(timezone) => ("timedatectl", vec!["set-timezone".to_owned(), timezone.clone()]),
            Request::SetNetworkTime(enabled) => ("timedatectl", vec!["set-ntp".to_owned(), enabled.to_string()]),
            Request::SetLocalClock(enabled) => ("timedatectl", vec!["set-local-rtc".to_owned(), enabled.to_string()]),
        }
    }
}

pub fn perform(commands: &dyn CommandRunner, request: &Request) -> Result<(), CommandError> {
    let (program, arguments) = request.command();
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    commands.run(program, &arguments).map(drop)
}
