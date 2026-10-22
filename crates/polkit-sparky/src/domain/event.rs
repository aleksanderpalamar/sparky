#[derive(Debug, Clone)]
pub enum Event {
    Prompt { text: String, echo: bool },
    Info(String),
    Problem(String),
    Completed(bool),
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Authorized,
    Cancelled,
    Denied,
}

impl Outcome {
    pub fn code(self) -> u8 {
        match self {
            Self::Authorized => 0,
            Self::Cancelled => 1,
            Self::Denied => 2,
        }
    }

    pub fn from_code(code: Option<i32>) -> Self {
        match code {
            Some(0) => Self::Authorized,
            Some(1) => Self::Cancelled,
            _ => Self::Denied,
        }
    }
}
