use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use iced::Task;

use crate::app::background::background;
use crate::app::jobs::{Job, JobEvent, Origin, Work};
use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::archive::{Extraction, extractable};
use crate::domain::transfer::{Ending, Progress};
use crate::domain::undo::Record;
use crate::i18n;
use crate::ports::Archiver;

const INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone)]
pub struct ExtractReport {
    pub done: Vec<PathBuf>,
    pub ending: Ending,
}

impl Files {
    pub fn extract(&mut self) -> Task<Message> {
        let archives = self.archives();
        if archives.is_empty() {
            return Task::none();
        }
        if self.job.is_some() {
            self.fail(i18n::text("job-busy"));
            return Task::none();
        }
        let cancel = Arc::new(AtomicBool::new(false));
        self.job = Some(Job {
            cancel: cancel.clone(),
            progress: Progress::default(),
            work: Work::Extract,
            origin: Origin::User,
        });
        self.listing.clear_marks();
        let archiver = self.services.archiver.clone();
        let stream = background(move |send| {
            let report = extract_all(&archives, archiver.as_ref(), &cancel, send);
            send(JobEvent::Extracted(report));
        });
        Task::run(stream, Message::Job)
    }

    pub(super) fn extracted(&mut self, report: ExtractReport) {
        self.refresh();
        for path in &report.done {
            self.journal.push(Record::Created(path.clone()));
        }
        let last = report
            .done
            .last()
            .filter(|last| last.parent() == Some(&self.location));
        if let Some(name) = last.and_then(|last| last.file_name()) {
            self.listing.select_name(&name.to_string_lossy());
        }
        let count = report.done.len();
        match (&report.ending, report.done.as_slice()) {
            (Ending::Finished, [only]) => self.inform(i18n::with(
                "extracted-into",
                "name",
                only.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            )),
            (Ending::Finished, _) => self.inform(i18n::counted("extracted-many", count)),
            (Ending::Cancelled, _) => self.inform(i18n::counted("job-cancelled", count)),
            (Ending::Failed { name, error }, _) => self.fail(format!("{name}: {error}")),
        }
    }

    fn archives(&self) -> Vec<PathBuf> {
        self.listing
            .chosen()
            .into_iter()
            .filter(|name| {
                self.listing
                    .find(name)
                    .is_some_and(|entry| !entry.is_directory())
            })
            .filter(|name| extractable(name).is_some())
            .map(|name| self.location.join(name))
            .collect()
    }
}

fn extract_all(
    archives: &[PathBuf],
    archiver: &dyn Archiver,
    cancel: &AtomicBool,
    send: &dyn Fn(JobEvent),
) -> ExtractReport {
    let mut done = Vec::new();
    let mut last = Instant::now()
        .checked_sub(INTERVAL)
        .unwrap_or_else(Instant::now);
    for archive in archives {
        let into = archive.parent().unwrap_or(Path::new("/"));
        let result = archiver.extract(archive, into, &mut |progress| {
            if last.elapsed() >= INTERVAL || progress.items_done == progress.items_total {
                send(JobEvent::Progress(progress));
                last = Instant::now();
            }
            !cancel.load(Ordering::SeqCst)
        });
        let ending = match result {
            Ok(Extraction::Done(path)) => {
                done.push(path);
                continue;
            }
            Ok(Extraction::Cancelled) => Ending::Cancelled,
            Err(error) => Ending::Failed {
                name: archive
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                error: error.to_string(),
            },
        };
        return ExtractReport { done, ending };
    }
    ExtractReport {
        done,
        ending: Ending::Finished,
    }
}
