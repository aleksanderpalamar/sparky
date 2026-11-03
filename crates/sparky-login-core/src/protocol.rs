use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

pub const SOCKET_VARIABLE: &str = "SPARKY_LOGIN_SOCKET";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Power {
    Shutdown,
    Reboot,
    Suspend,
}

impl Power {
    pub const ALL: [Power; 3] = [Power::Shutdown, Power::Reboot, Power::Suspend];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "reason", rename_all = "kebab-case")]
pub enum Refusal {
    Denied,
    Unavailable(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "kebab-case")]
pub enum Request {
    Hello,
    Login {
        username: String,
        password: Zeroizing<String>,
        session: String,
    },
    Power {
        action: Power,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "kebab-case")]
pub enum Reply {
    Welcome {
        last_user: Option<String>,
        last_session: Option<String>,
    },
    LoggedIn,
    Refused {
        refusal: Refusal,
    },
    Done,
    Failed {
        message: String,
    },
}
