use std::path::Path;

use crate::message::{CaptureArea, Reply, Request, SessionAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    Missing,
    Unknown(String),
    InvalidWorkspace(String),
    InvalidNumber(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::Missing => {
                write!(
                    formatter,
                    "uso: sparky-msg workspaces | outputs | focus-workspace <n> | subscribe | reload | capture output [monitor] <arquivo> | capture window <arquivo> | capture region <x> <y> <largura> <altura> <arquivo> | session lock|suspend|logout|reboot|poweroff"
                )
            }
            CommandError::Unknown(name) => write!(formatter, "comando desconhecido: {name}"),
            CommandError::InvalidWorkspace(value) => write!(formatter, "área inválida: {value}"),
            CommandError::InvalidNumber(value) => write!(formatter, "número inválido: {value}"),
        }
    }
}

pub fn parse(arguments: &[String]) -> Result<Request, CommandError> {
    let Some(name) = arguments.first() else {
        return Err(CommandError::Missing);
    };
    match name.as_str() {
        "workspaces" => Ok(Request::Workspaces),
        "subscribe" => Ok(Request::Subscribe),
        "reload" => Ok(Request::ReloadConfig),
        "outputs" => Ok(Request::Outputs),
        "info" => Ok(Request::Info),
        "capture" => capture(&arguments[1..]),
        "focus-workspace" => workspace(arguments.get(1)),
        "session" => session(arguments.get(1)),
        other => Err(CommandError::Unknown(other.to_owned())),
    }
}

fn session(name: Option<&String>) -> Result<Request, CommandError> {
    let name = name.ok_or(CommandError::Missing)?;
    let action = SessionAction::named(name).ok_or_else(|| CommandError::Unknown(name.clone()))?;
    Ok(Request::Session { action })
}

pub fn with_absolute_path(request: Request, directory: &Path) -> Request {
    match request {
        Request::Capture { area, path } => Request::Capture {
            path: directory.join(path).to_string_lossy().into_owned(),
            area,
        },
        other => other,
    }
}

pub fn succeeded(reply: &Reply) -> bool {
    !matches!(reply, Reply::Error { .. })
}

fn capture(arguments: &[String]) -> Result<Request, CommandError> {
    let texts: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let (area, path) = match texts.as_slice() {
        ["output", path] => (CaptureArea::Output { output: None }, path),
        ["output", name, path] => (
            CaptureArea::Output {
                output: Some((*name).to_owned()),
            },
            path,
        ),
        ["window", path] => (CaptureArea::Window, path),
        ["region", x, y, width, height, path] => (region(x, y, width, height)?, path),
        _ => return Err(CommandError::Missing),
    };
    Ok(Request::Capture {
        area,
        path: (*path).to_owned(),
    })
}

fn region(x: &str, y: &str, width: &str, height: &str) -> Result<CaptureArea, CommandError> {
    let number = |text: &str| {
        text.parse::<i32>()
            .map_err(|_| CommandError::InvalidNumber(text.to_owned()))
    };
    Ok(CaptureArea::Region {
        output: None,
        x: number(x)?,
        y: number(y)?,
        width: number(width)?,
        height: number(height)?,
    })
}

fn workspace(value: Option<&String>) -> Result<Request, CommandError> {
    let value = value.ok_or(CommandError::Missing)?;
    value
        .parse()
        .map(|id| Request::FocusWorkspace { id })
        .map_err(|_| CommandError::InvalidWorkspace(value.clone()))
}
