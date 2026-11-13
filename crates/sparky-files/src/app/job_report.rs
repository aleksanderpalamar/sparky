use crate::app::jobs::Origin;
use crate::app::state::Files;
use crate::domain::clip::Clip;
use crate::domain::transfer::{Action, Ending, Outcome};
use crate::domain::undo::Record;
use crate::i18n;

impl Files {
    pub(super) fn transferred(&mut self, action: Action, origin: Origin, outcome: Outcome) {
        self.refresh();
        if action == Action::Move {
            let moves: Vec<_> = outcome
                .completed
                .iter()
                .map(|step| (step.source.clone(), step.target.clone()))
                .collect();
            let moved = self.services.sidebar.relocated(&moves);
            if let Err(error) = moved {
                self.fail(error);
                return;
            }
        }
        if outcome.ending == Ending::Finished {
            self.release_cut(&origin);
        }
        if !matches!(origin, Origin::Undo(_)) && !outcome.completed.is_empty() {
            self.journal.push(Record::Transferred {
                action,
                steps: outcome.completed.clone(),
                displaced: outcome.displaced.clone(),
            });
        }
        let last = outcome.completed.last().map(|step| step.target.clone());
        if let Some(last) = last.filter(|last| last.parent() == Some(self.location.as_path()))
            && let Some(name) = last.file_name().and_then(|name| name.to_str())
        {
            self.listing.select_name(name);
        }
        match origin {
            Origin::User | Origin::Pasted(_) | Origin::Dropped => self.report(action, &outcome),
            Origin::Undo(displaced) => self.report_undo(&displaced, &outcome),
        }
    }

    fn release_cut(&mut self, origin: &Origin) {
        let Origin::Pasted(cut @ Clip::Cut(_)) = origin else {
            return;
        };
        if self.clipboard_clip().as_ref() != Some(cut) {
            return;
        }
        self.clip = None;
        let _ = self.services.clipboard.clear();
    }

    fn report(&mut self, action: Action, outcome: &Outcome) {
        let done = outcome.completed.len();
        match &outcome.ending {
            Ending::Failed { name, error } => self.fail(format!("{name}: {error}")),
            Ending::Cancelled => self.inform(i18n::counted("job-cancelled", done)),
            Ending::Finished => {
                let key = match action {
                    Action::Copy => "job-copied",
                    Action::Move => "job-moved",
                };
                let mut text = i18n::counted(key, done);
                if outcome.skipped > 0 {
                    text = format!("{text} · {}", i18n::counted("job-skipped", outcome.skipped));
                }
                self.inform(text);
            }
        }
    }
}
