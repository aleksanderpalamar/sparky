use sparky_ipc::SessionAction;

use crate::domain::app::{App, Runs};

const ENTRIES: [(SessionAction, &str, &str, &[&str]); 5] = [
    (SessionAction::Lock, "Bloquear", "Tranca a tela", &["lock", "trancar", "tela"]),
    (SessionAction::Suspend, "Suspender", "Tranca a tela e suspende o computador", &["suspend", "dormir"]),
    (SessionAction::Logout, "Sair da sessão", "Fecha os aplicativos e encerra o sparky", &["logout", "encerrar"]),
    (SessionAction::Reboot, "Reiniciar", "Fecha os aplicativos e reinicia o computador", &["reboot", "restart"]),
    (SessionAction::Poweroff, "Desligar", "Fecha os aplicativos e desliga o computador", &["poweroff", "shutdown"]),
];

pub fn entries() -> Vec<App> {
    ENTRIES
        .iter()
        .map(|(action, name, detail, keywords)| App {
            id: format!("session-{name}"),
            name: (*name).to_owned(),
            detail: Some((*detail).to_owned()),
            keywords: keywords.iter().map(|word| (*word).to_owned()).collect(),
            command: String::new(),
            runs: Runs::Session(*action),
        })
        .collect()
}
