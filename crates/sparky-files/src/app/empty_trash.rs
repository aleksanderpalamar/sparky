use iced::Task;
use iced::keyboard::Key;

use crate::app::keys::{self, Answer};
use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::i18n;
use crate::ports::EmptyReport;

impl Files {
    pub fn ask_empty_trash(&mut self) {
        if self.emptying {
            return;
        }
        let Some(trash) = self.trash_here() else {
            return;
        };
        match trash.names() {
            Ok(names) if names.is_empty() => self.inform(i18n::text("empty-trash-state")),
            Ok(names) => self.mode = Mode::ConfirmEmptyTrash(names.len()),
            Err(error) => self.fail(error),
        }
    }

    pub fn confirm(&mut self, key: &Key) -> Task<Message> {
        let Some(answer) = keys::confirm(key) else {
            return Task::none();
        };
        match self.mode {
            Mode::ConfirmRemoval(..) => self.answer_removal(answer),
            Mode::ConfirmEmptyTrash(_) => return self.answer_empty_trash(answer),
            _ => {}
        }
        Task::none()
    }

    fn answer_empty_trash(&mut self, answer: Answer) -> Task<Message> {
        self.mode = Mode::Browse;
        if answer == Answer::No || self.emptying {
            return Task::none();
        }
        let Some(trash) = self.trash_here() else {
            return Task::none();
        };
        self.emptying = true;
        self.inform(i18n::text("emptying-trash"));
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || trash.empty())
                    .await
                    .map_err(|error| error.to_string())
                    .and_then(|result| result.map_err(|error| error.to_string()))
            },
            Message::TrashEmptied,
        )
    }

    pub fn trash_emptied(&mut self, result: Result<EmptyReport, String>) {
        if !self.emptying {
            return;
        }
        self.emptying = false;
        if self.in_trash() {
            self.refresh();
        }
        match result {
            Ok(report) if report.errors.is_empty() => {
                self.inform(format!("{} itens removidos", report.removed));
            }
            Ok(report) => self.fail(format!(
                "{} itens removidos; {} falhas. {}",
                report.removed,
                report.errors.len(),
                report.errors[0]
            )),
            Err(error) => self.fail(error),
        }
    }
}
