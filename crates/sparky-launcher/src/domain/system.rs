use sparky_ipc::SessionAction;

use crate::domain::app::{App, Runs};
use crate::domain::icon::Icon;
use crate::i18n;

struct Entry {
    action: SessionAction,
    id: &'static str,
    name: &'static str,
    detail: &'static str,
    keywords: &'static [&'static str],
    icon: &'static [&'static str],
}

const ENTRIES: [Entry; 5] = [
    Entry {
        action: SessionAction::Lock,
        id: "lock",
        name: "lock-name",
        detail: "lock-detail",
        keywords: &["lock", "trancar", "tela"],
        icon: &["system-lock-screen-symbolic"],
    },
    Entry {
        action: SessionAction::Suspend,
        id: "suspend",
        name: "suspend-name",
        detail: "suspend-detail",
        keywords: &["suspend", "dormir"],
        icon: &[
            "system-suspend-symbolic",
            "weather-clear-night-symbolic",
            "media-playback-pause-symbolic",
        ],
    },
    Entry {
        action: SessionAction::Logout,
        id: "logout",
        name: "logout-name",
        detail: "logout-detail",
        keywords: &["logout", "encerrar"],
        icon: &["system-log-out-symbolic"],
    },
    Entry {
        action: SessionAction::Reboot,
        id: "reboot",
        name: "reboot-name",
        detail: "reboot-detail",
        keywords: &["reboot", "restart"],
        icon: &["system-reboot-symbolic", "view-refresh-symbolic"],
    },
    Entry {
        action: SessionAction::Poweroff,
        id: "poweroff",
        name: "poweroff-name",
        detail: "poweroff-detail",
        keywords: &["poweroff", "shutdown"],
        icon: &["system-shutdown-symbolic"],
    },
];

pub fn entries() -> Vec<App> {
    ENTRIES
        .iter()
        .map(|entry| App {
            id: format!("session-{}", entry.id),
            name: i18n::text(entry.name),
            detail: Some(i18n::text(entry.detail)),
            keywords: entry
                .keywords
                .iter()
                .map(|word| (*word).to_owned())
                .collect(),
            command: String::new(),
            runs: Runs::Session(entry.action),
            icon: Some(Icon::Symbolic(entry.icon)),
            categories: Vec::new(),
        })
        .collect()
}
