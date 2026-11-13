use iced::Task;

use crate::app::jobs::Origin;
use crate::app::message::Message;
use crate::app::state::Files;
use crate::batch_rename::rename_all;
use crate::domain::transfer::{Action, Ending, Outcome};
use crate::domain::undo::{Record, reversed};
use crate::i18n;
use crate::operations::Planned;

impl Files {
    pub fn undo(&mut self) -> Task<Message> {
        if self.job.is_some() {
            self.fail(i18n::text("job-busy"));
            return Task::none();
        }
        let Some(record) = self.journal.pop() else {
            self.inform(i18n::text("nothing-to-undo"));
            return Task::none();
        };
        let disk = self.services.disk.as_ref();
        let result = match record {
            Record::Transferred {
                action: Action::Move,
                steps,
            } => {
                let planned = Planned {
                    steps: reversed(&steps),
                    skipped: 0,
                };
                return self.run_job(planned, Action::Move, Origin::Undo);
            }
            Record::Transferred {
                action: Action::Copy,
                steps,
            } => steps
                .iter()
                .rev()
                .try_for_each(|step| disk.trash(&step.target).map(drop)),
            Record::Renamed { from, to } => disk.relocate(&to, &from).and_then(|()| {
                self.services
                    .sidebar
                    .relocated(&[(to, from)])
                    .map_err(std::io::Error::other)
            }),
            Record::RenamedMany(pairs) => {
                let back: Vec<_> = pairs
                    .iter()
                    .map(|(from, to)| (to.clone(), from.clone()))
                    .collect();
                rename_all(&back, disk)
                    .map_err(|failure| std::io::Error::other(failure.to_string()))
                    .and_then(|()| {
                        self.services
                            .sidebar
                            .relocated(&back)
                            .map_err(std::io::Error::other)
                    })
            }
            Record::Trashed(entries) => entries
                .iter()
                .rev()
                .try_for_each(|entry| disk.untrash(entry).map(drop)),
            Record::Created(path) => disk.trash(&path).map(drop),
        };
        self.refresh_trashes();
        self.refresh();
        match result {
            Ok(()) => self.inform(i18n::text("undone")),
            Err(error) => self.fail(i18n::with("undo-failed", "error", error.to_string())),
        }
        Task::none()
    }

    pub(super) fn report_undo(&mut self, outcome: &Outcome) {
        match &outcome.ending {
            Ending::Finished => self.inform(i18n::text("undone")),
            Ending::Cancelled => {
                self.inform(i18n::counted("job-cancelled", outcome.completed.len()))
            }
            Ending::Failed { name, error } => self.fail(i18n::with(
                "undo-failed",
                "error",
                format!("{name}: {error}"),
            )),
        }
    }
}
