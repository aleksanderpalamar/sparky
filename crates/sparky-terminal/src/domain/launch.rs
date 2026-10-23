use std::fmt;
use std::path::{Path, PathBuf};

use crate::domain::zsh::ZSH;

pub const ENVIRONMENT: [(&str, &str); 3] = [
    ("TERM", "xterm-256color"),
    ("COLORTERM", "truecolor"),
    ("TERM_PROGRAM", "sparky-terminal"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Program {
    Shell,
    Command { program: String, args: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub program: Program,
    pub directory: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    Installed,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub program: Option<(String, Vec<String>)>,
    pub environment: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchError {
    MissingCommand,
    MissingDirectory,
    Unknown(String),
}

impl fmt::Display for LaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LaunchError::MissingCommand => formatter.write_str("-e precisa de um comando"),
            LaunchError::MissingDirectory => formatter.write_str("-d precisa de uma pasta"),
            LaunchError::Unknown(option) => write!(formatter, "opção desconhecida: {option}"),
        }
    }
}

pub fn parse(arguments: &[String]) -> Result<Launch, LaunchError> {
    let mut launch = Launch {
        program: Program::Shell,
        directory: None,
    };
    let mut rest = arguments.iter();
    while let Some(argument) = rest.next() {
        match argument.as_str() {
            "-e" | "--command" => {
                let program = rest.next().ok_or(LaunchError::MissingCommand)?.clone();
                launch.program = Program::Command {
                    program,
                    args: rest.cloned().collect(),
                };
                return Ok(launch);
            }
            "-d" | "--working-directory" => {
                launch.directory = Some(PathBuf::from(
                    rest.next().ok_or(LaunchError::MissingDirectory)?,
                ));
            }
            other => return Err(LaunchError::Unknown(other.to_owned())),
        }
    }
    Ok(launch)
}

pub fn plan(launch: &Launch, zsh: Availability, zdotdir: &Path) -> Plan {
    let program = match (&launch.program, zsh) {
        (Program::Shell, Availability::Installed) => Some((ZSH.to_owned(), Vec::new())),
        (Program::Shell, Availability::Missing) => None,
        (Program::Command { program, args }, _) => Some((program.clone(), args.clone())),
    };
    let mut environment: Vec<(String, String)> = ENVIRONMENT
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect();
    environment.push(("ZDOTDIR".to_owned(), zdotdir.display().to_string()));
    Plan {
        program,
        environment,
    }
}
