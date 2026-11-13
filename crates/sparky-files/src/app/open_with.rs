use std::path::PathBuf;

use iced::Task;
use iced::keyboard::Key;
use iced::keyboard::key::Named;
use iced::widget::operation;

use crate::app::message::Message;
use crate::app::state::{Files, OpenWith};
use crate::domain::application::Remember;
use crate::i18n;

pub const SEARCH: &str = "open-with-search";

impl Files {
    pub fn begin_open_with(&mut self, path: PathBuf) -> Task<Message> {
        let choices = self.services.opener.applications(&path);
        if choices.is_empty() {
            self.fail(i18n::text("no-app"));
            return Task::none();
        }
        let selected = choices.first("");
        self.open_with = Some(OpenWith {
            path,
            choices,
            query: String::new(),
            selected,
            remember: Remember::Once,
        });
        operation::focus(SEARCH)
    }

    pub fn open_with_query(&mut self, query: String) {
        let Some(window) = self.open_with.as_mut() else {
            return;
        };
        window.query = query;
        let visible = window.choices.visible(&window.query);
        let kept = visible
            .iter()
            .any(|(_, app)| Some(&app.id) == window.selected.as_ref());
        if !kept {
            window.selected = window.choices.first(&window.query);
        }
    }

    pub fn open_with_select(&mut self, id: String) {
        if let Some(window) = self.open_with.as_mut()
            && window.choices.find(&id).is_some()
        {
            window.selected = Some(id);
        }
    }

    pub fn open_with_remember(&mut self, always: bool) {
        if let Some(window) = self.open_with.as_mut() {
            window.remember = match always {
                true => Remember::Always,
                false => Remember::Once,
            };
        }
    }

    pub fn open_with_confirm(&mut self) {
        let selected = self
            .open_with
            .as_ref()
            .and_then(|window| window.selected.clone());
        if let Some(id) = selected {
            self.open_with_choice(&id);
        }
    }

    pub fn open_with_choice(&mut self, id: &str) {
        let known = self
            .open_with
            .as_ref()
            .is_some_and(|window| window.choices.find(id).is_some());
        if !known {
            return;
        }
        let Some(window) = self.open_with.take() else {
            return;
        };
        let opener = self.services.opener.as_ref();
        if let Err(error) = opener.open_with(&window.path, id, window.remember) {
            self.fail(error);
        }
    }

    pub fn open_with_key(&mut self, key: &Key) {
        let forward = match key.as_ref() {
            Key::Named(Named::ArrowDown) => true,
            Key::Named(Named::ArrowUp) => false,
            Key::Named(Named::Enter) => return self.open_with_confirm(),
            _ => return,
        };
        if let Some(window) = self.open_with.as_mut() {
            window.selected =
                window
                    .choices
                    .step(&window.query, window.selected.as_deref(), forward);
        }
    }
}
