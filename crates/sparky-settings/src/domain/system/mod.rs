pub mod datetime;
pub mod key_value;
pub mod localectl;
pub mod power;
pub mod profiles;
pub mod region;
pub mod users;
pub mod passwd;
pub mod media;
pub mod volumes;
pub mod themes;

#[cfg(test)]
mod datetime_tests;
#[cfg(test)]
mod power_tests;
#[cfg(test)]
mod region_tests;
#[cfg(test)]
mod users_tests;
#[cfg(test)]
mod media_tests;
#[cfg(test)]
mod themes_tests;

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
    Locale,
    ConsoleKeymap,
    SystemLayout,
    Volume(usize),
    MediaStatus,
    Theme(usize),
    ThemeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: RowId,
    pub label: String,
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
    SetLocale(String),
    SetConsoleKeymap(String),
    SetSystemLayout(String),
    MountVolume(String),
    UnmountVolume(String),
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
            Request::SetLocale(locale) => ("localectl", vec!["set-locale".to_owned(), format!("LANG={locale}")]),
            Request::SetConsoleKeymap(keymap) => ("localectl", vec!["set-keymap".to_owned(), keymap.clone()]),
            Request::SetSystemLayout(layout) => ("localectl", vec!["set-x11-keymap".to_owned(), layout.clone()]),
            Request::MountVolume(path) => ("udisksctl", vec!["mount".to_owned(), "-b".to_owned(), path.clone()]),
            Request::UnmountVolume(path) => ("udisksctl", vec!["unmount".to_owned(), "-b".to_owned(), path.clone()]),
        }
    }
}

pub fn perform(commands: &dyn CommandRunner, request: &Request) -> Result<(), CommandError> {
    let (program, arguments) = request.command();
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    commands.run(program, &arguments).map(drop)
}
