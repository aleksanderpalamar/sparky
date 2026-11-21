use sparky_xdg::desktop_entry::DesktopEntry;

use super::exec::arguments;

const DESKTOP: &str = "sparky";
const SUFFIX: &str = ".desktop";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub arguments: Vec<String>,
    pub terminal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub launch: Launch,
    pub window_class: Option<String>,
}

impl App {
    pub fn of(id: &str, entry: &DesktopEntry, locale: &str) -> Option<Self> {
        if entry.no_display || !entry.shown_in(DESKTOP) {
            return None;
        }
        let name = entry.name.get(locale)?.trim().to_owned();
        let arguments = arguments(entry.exec.as_deref()?)?;
        (!name.is_empty()).then(|| Self {
            id: id.strip_suffix(SUFFIX).unwrap_or(id).to_owned(),
            name,
            icon: entry.icon.clone().filter(|icon| !icon.is_empty()),
            launch: Launch {
                arguments,
                terminal: entry.terminal,
            },
            window_class: entry.startup_wm_class.clone(),
        })
    }

    pub fn owns(&self, app_id: &str) -> bool {
        let wanted = app_id.to_lowercase();
        self.names().any(|name| name.to_lowercase() == wanted)
    }

    fn names(&self) -> impl Iterator<Item = &str> {
        let program = self
            .launch
            .arguments
            .first()
            .and_then(|path| path.rsplit('/').next());
        let reverse = self.id.rsplit('.').next();
        [
            Some(self.id.as_str()),
            self.window_class.as_deref(),
            program,
            reverse,
        ]
        .into_iter()
        .flatten()
    }
}
