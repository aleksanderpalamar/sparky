use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sparky_login_core::protocol::Refusal;
use sparky_login_core::sessions::Session;
use zeroize::Zeroizing;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "job", rename_all = "kebab-case")]
pub enum Job {
    Greeter {
        user: String,
        command: Vec<String>,
        vt: u32,
        socket: PathBuf,
    },
    Login {
        username: String,
        password: Zeroizing<String>,
        session: Session,
        vt: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "order", rename_all = "kebab-case")]
pub enum Order {
    Start,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "report", rename_all = "kebab-case")]
pub enum Report {
    Authenticated,
    Refused { refusal: Refusal },
    Running,
    Ended,
    Failed { message: String },
}
