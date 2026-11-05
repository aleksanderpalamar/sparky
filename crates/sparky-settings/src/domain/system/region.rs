use crate::domain::system::localectl;
use crate::domain::system::{Change, Control, Request, Row, RowId};
use crate::domain::value::Step;
use crate::ports::commands::CommandRunner;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Region {
    locale: Option<String>,
    keymap: Option<String>,
    layout: Option<String>,
    locales: Vec<String>,
    keymaps: Vec<String>,
    layouts: Vec<String>,
}

impl Region {
    pub fn load(commands: &dyn CommandRunner) -> Self {
        let status = commands.run("localectl", &["status"]).unwrap_or_default();
        Self {
            locale: localectl::locale(&status),
            keymap: localectl::field(&status, "VC Keymap"),
            layout: localectl::field(&status, "X11 Layout"),
            locales: options(commands, "list-locales"),
            keymaps: options(commands, "list-keymaps"),
            layouts: options(commands, "list-x11-keymap-layouts"),
        }
    }

    pub fn rows(&self) -> Vec<Row> {
        [
            (RowId::Locale, "row-system-language"),
            (RowId::ConsoleKeymap, "row-console-keyboard"),
            (RowId::SystemLayout, "row-system-layout"),
        ]
        .into_iter()
        .filter_map(|(id, label)| {
            let (current, choices) = self.selection(id)?;
            Some(Row {
                id,
                label: crate::i18n::text(label),
                value: current
                    .clone()
                    .unwrap_or_else(|| crate::i18n::text("unset")),
                control: match choices.is_empty() {
                    true => Control::Reading,
                    false => Control::Choice(choices.to_vec()),
                },
            })
        })
        .collect()
    }

    pub fn request(&self, id: RowId, change: Change) -> Option<Request> {
        let (current, choices) = self.selection(id)?;
        let value = match change {
            Change::Text(input) if self.unknown_value(id, &input).is_none() => {
                input.trim().to_owned()
            }
            Change::Step(step) => neighbor(current.as_deref(), choices, step)?,
            _ => return None,
        };
        match id {
            RowId::Locale => Some(Request::SetLocale(value)),
            RowId::ConsoleKeymap => Some(Request::SetConsoleKeymap(value)),
            RowId::SystemLayout => Some(Request::SetSystemLayout(value)),
            _ => None,
        }
    }

    pub fn unknown_value(&self, id: RowId, input: &str) -> Option<String> {
        let (_, choices) = self.selection(id)?;
        (!choices.iter().any(|choice| choice == input.trim())).then(|| input.trim().to_owned())
    }

    fn selection(&self, id: RowId) -> Option<(&Option<String>, &[String])> {
        match id {
            RowId::Locale => Some((&self.locale, &self.locales)),
            RowId::ConsoleKeymap => Some((&self.keymap, &self.keymaps)),
            RowId::SystemLayout => Some((&self.layout, &self.layouts)),
            _ => None,
        }
    }
}

fn options(commands: &dyn CommandRunner, command: &str) -> Vec<String> {
    localectl::list(&commands.run("localectl", &[command]).unwrap_or_default())
}

fn neighbor(current: Option<&str>, choices: &[String], step: Step) -> Option<String> {
    if choices.is_empty() {
        return None;
    }
    let index = choices
        .iter()
        .position(|choice| Some(choice.as_str()) == current);
    let next = match (index, step) {
        (None, _) => 0,
        (Some(0), Step::Less) => choices.len() - 1,
        (Some(index), Step::Less) => index - 1,
        (Some(index), Step::More) => (index + 1) % choices.len(),
    };
    choices
        .get(next)
        .filter(|choice| Some(choice.as_str()) != current)
        .cloned()
}
