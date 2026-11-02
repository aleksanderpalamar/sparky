use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sparky_login_core::sessions::Session;

pub const SEAT: &str = "seat0";
const DEFAULT_PATH: &str = "/usr/local/bin:/usr/bin:/bin";
const FALLBACK_SHELL: &str = "/bin/sh";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    pub home: PathBuf,
    pub shell: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    User,
    Greeter,
}

impl Class {
    fn name(self) -> &'static str {
        match self {
            Class::User => "user",
            Class::Greeter => "greeter",
        }
    }
}

pub fn tty(vt: u32) -> PathBuf {
    PathBuf::from(format!("/dev/tty{vt}"))
}

pub fn pam_variables(class: Class, vt: u32, session: Option<&Session>) -> Vec<(String, String)> {
    let mut variables = vec![
        pair("XDG_SESSION_TYPE", "wayland"),
        pair("XDG_SESSION_CLASS", class.name()),
        pair("XDG_SEAT", SEAT),
        pair("XDG_VTNR", &vt.to_string()),
    ];
    if let Some(session) = session {
        variables.push(pair("XDG_SESSION_DESKTOP", &session.id));
        if !session.desktop_names.is_empty() {
            variables.push(pair(
                "XDG_CURRENT_DESKTOP",
                &session.desktop_names.join(":"),
            ));
        }
    }
    variables
}

pub fn environment(account: &Account, pam: Vec<(String, String)>) -> Vec<(String, String)> {
    let home = account.home.to_string_lossy();
    let shell = account.shell.to_string_lossy();
    let base = [
        pair("HOME", &home),
        pair("USER", &account.name),
        pair("LOGNAME", &account.name),
        pair("SHELL", &shell),
        pair("PATH", DEFAULT_PATH),
    ];
    let merged: BTreeMap<String, String> = base.into_iter().chain(pam).collect();
    merged.into_iter().collect()
}

pub fn login_command(account: &Account, exec: &str) -> Vec<String> {
    let shell = match account.shell.as_os_str().is_empty() {
        true => FALLBACK_SHELL.to_owned(),
        false => account.shell.to_string_lossy().into_owned(),
    };
    vec![
        shell,
        "-l".to_owned(),
        "-c".to_owned(),
        format!("exec {exec}"),
    ]
}

fn pair(name: &str, value: &str) -> (String, String) {
    (name.to_owned(), value.to_owned())
}
