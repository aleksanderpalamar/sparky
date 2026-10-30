#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Auto,
    Manual,
}

impl Method {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "auto" => Some(Method::Auto),
            "manual" => Some(Method::Manual),
            _ => None,
        }
    }

    pub(super) fn keyword(self) -> &'static str {
        match self {
            Method::Auto => "auto",
            Method::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoDns {
    Use,
    Ignore,
}

impl AutoDns {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "no" => Some(AutoDns::Use),
            "yes" => Some(AutoDns::Ignore),
            _ => None,
        }
    }

    pub(super) fn keyword(self) -> &'static str {
        match self {
            AutoDns::Use => "no",
            AutoDns::Ignore => "yes",
        }
    }
}
