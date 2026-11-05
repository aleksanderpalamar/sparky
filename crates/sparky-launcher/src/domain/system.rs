use sparky_ipc::SessionAction;

use crate::domain::app::{App, Runs};
use crate::i18n;

const ENTRIES: [(SessionAction, &str, &str, &str, &[&str]); 5] = [
    (
        SessionAction::Lock,
        "lock",
        "lock-name",
        "lock-detail",
        &["lock", "trancar", "tela"],
    ),
    (
        SessionAction::Suspend,
        "suspend",
        "suspend-name",
        "suspend-detail",
        &["suspend", "dormir"],
    ),
    (
        SessionAction::Logout,
        "logout",
        "logout-name",
        "logout-detail",
        &["logout", "encerrar"],
    ),
    (
        SessionAction::Reboot,
        "reboot",
        "reboot-name",
        "reboot-detail",
        &["reboot", "restart"],
    ),
    (
        SessionAction::Poweroff,
        "poweroff",
        "poweroff-name",
        "poweroff-detail",
        &["poweroff", "shutdown"],
    ),
];

pub fn entries() -> Vec<App> {
    ENTRIES
        .iter()
        .map(|(action, id, name, detail, keywords)| App {
            id: format!("session-{id}"),
            name: i18n::text(name),
            detail: Some(i18n::text(detail)),
            keywords: keywords.iter().map(|word| (*word).to_owned()).collect(),
            command: String::new(),
            runs: Runs::Session(*action),
        })
        .collect()
}
