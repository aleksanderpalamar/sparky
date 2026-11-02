use std::path::PathBuf;

use sparky_login_core::appearance;

pub const SOCKET: &str = "/run/sparky-login/greeter.sock";
pub const MEMORY: &str = "/var/lib/sparky-login/memory.json";
pub const GREETER_USER: &str = "sparky-login";
pub const GREETER_COMMAND: [&str; 3] = ["sparky", "--login-screen", "sparky-login-greeter"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub vt: u32,
    pub socket: PathBuf,
    pub memory: PathBuf,
    pub greeter_user: String,
    pub greeter_command: Vec<String>,
    pub worker: PathBuf,
    pub data_dirs: Option<String>,
    pub appearance: PathBuf,
}

impl Settings {
    pub fn new(vt: u32, worker: PathBuf) -> Self {
        Self {
            vt,
            socket: PathBuf::from(SOCKET),
            memory: PathBuf::from(MEMORY),
            greeter_user: GREETER_USER.to_owned(),
            greeter_command: GREETER_COMMAND
                .iter()
                .map(|part| (*part).to_owned())
                .collect(),
            worker,
            data_dirs: std::env::var("XDG_DATA_DIRS").ok(),
            appearance: PathBuf::from(appearance::FOLDER),
        }
    }
}
