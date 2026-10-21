use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    Unavailable,
    Rejected { code: Option<i32>, message: String },
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::Unavailable => write!(formatter, "programa não encontrado"),
            CommandError::Rejected { message, .. } if !message.is_empty() => write!(formatter, "{message}"),
            CommandError::Rejected { code, .. } => write!(formatter, "terminou com erro {code:?}"),
        }
    }
}

pub trait CommandRunner {
    fn run(&self, program: &str, arguments: &[&str]) -> Result<String, CommandError>;
}

pub fn invocation(program: &str, arguments: &[&str]) -> String {
    match arguments.is_empty() {
        true => program.to_owned(),
        false => format!("{program} {}", arguments.join(" ")),
    }
}
