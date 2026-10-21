use sparky_ipc::SessionAction;
use sparky_xdg::desktop_entry::{DesktopEntry, command_of, list_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runs {
    Directly,
    InTerminal,
    Session(SessionAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub id: String,
    pub name: String,
    pub detail: Option<String>,
    pub keywords: Vec<String>,
    pub command: String,
    pub runs: Runs,
}

pub fn app_of(id: &str, entry: &DesktopEntry, locale: &str, desktop: &str) -> Option<App> {
    if entry.no_display || !entry.shown_in(desktop) {
        return None;
    }
    let name = entry.name.get(locale)?.to_owned();
    let command = command_of(entry.exec.as_deref()?);
    if command.is_empty() {
        return None;
    }
    let detail = entry
        .generic_name
        .get(locale)
        .or_else(|| entry.comment.get(locale))
        .filter(|detail| *detail != name)
        .map(str::to_owned);
    let runs = match entry.terminal {
        true => Runs::InTerminal,
        false => Runs::Directly,
    };
    Some(App {
        id: id.to_owned(),
        name,
        detail,
        keywords: list_of(entry.keywords.get(locale)),
        command,
        runs,
    })
}

pub fn command_line(app: &App, terminal: &str) -> Option<String> {
    match app.runs {
        Runs::Directly => Some(app.command.clone()),
        Runs::InTerminal => Some(format!("{terminal} -e {}", app.command)),
        Runs::Session(_) => None,
    }
}
