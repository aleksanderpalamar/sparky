use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use iced::Task;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};

use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::clip::Clip;
use crate::domain::transfer::{Action, Conflict, Outcome, Progress};
use crate::i18n;
use crate::operations::{self, Planned};
use crate::ports::Transfer;
use crate::transfer_job::execute;

#[derive(Debug, Clone)]
pub struct Job {
    pub cancel: Arc<AtomicBool>,
    pub progress: Progress,
    pub action: Action,
    pub clip: Clip,
    pub into: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    pub clip: Clip,
    pub into: PathBuf,
    pub names: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum JobEvent {
    Progress(Progress),
    Done(Outcome),
}

impl Files {
    pub fn paste(&mut self) -> Task<Message> {
        let Some(clip) = self.clip.clone() else {
            self.inform(i18n::text("nothing-to-paste"));
            return Task::none();
        };
        self.transfer(clip, self.location.clone())
    }

    pub fn transfer(&mut self, clip: Clip, into: PathBuf) -> Task<Message> {
        if self.job.is_some() {
            self.fail(i18n::text("job-busy"));
            return Task::none();
        }
        let names = operations::conflicts(&clip, &into, self.services.disk.as_ref());
        if !names.is_empty() {
            self.mode = Mode::Conflict(Pending { clip, into, names });
            return Task::none();
        }
        self.start_transfer(clip, into, Conflict::KeepBoth)
    }

    pub fn resolve_conflict(&mut self, choice: Conflict) -> Task<Message> {
        let Mode::Conflict(pending) = std::mem::replace(&mut self.mode, Mode::Browse) else {
            return Task::none();
        };
        self.start_transfer(pending.clip, pending.into, choice)
    }

    pub fn cancel_job(&mut self) -> bool {
        let Some(job) = &self.job else {
            return false;
        };
        job.cancel.store(true, Ordering::SeqCst);
        self.inform(i18n::text("job-cancelling"));
        true
    }

    pub fn job_event(&mut self, event: JobEvent) {
        match event {
            JobEvent::Progress(progress) => {
                if let Some(job) = &mut self.job {
                    job.progress = progress;
                }
            }
            JobEvent::Done(outcome) => {
                if let Some(job) = self.job.take() {
                    self.transferred(&job, outcome);
                }
            }
        }
    }

    fn start_transfer(&mut self, clip: Clip, into: PathBuf, conflict: Conflict) -> Task<Message> {
        let planned = match operations::plan(&clip, &into, self.services.disk.as_ref(), conflict) {
            Ok(planned) => planned,
            Err(failure) => {
                self.fail(failure);
                return Task::none();
            }
        };
        let cancel = Arc::new(AtomicBool::new(false));
        self.job = Some(Job {
            cancel: cancel.clone(),
            progress: Progress::default(),
            action: match clip {
                Clip::Copy(_) => Action::Copy,
                Clip::Cut(_) => Action::Move,
            },
            clip,
            into,
        });
        let transfer = self.services.transfer.clone();
        Task::run(run(planned, transfer, cancel), Message::Job)
    }
}

fn run(
    planned: Planned,
    transfer: Arc<dyn Transfer>,
    cancel: Arc<AtomicBool>,
) -> impl Stream<Item = JobEvent> {
    iced::stream::channel(16, async move |mut output| {
        let (sender, mut events) = mpsc::unbounded();
        thread::spawn(move || {
            let progress = sender.clone();
            let outcome = execute(
                planned.steps,
                planned.skipped,
                transfer.as_ref(),
                &cancel,
                &mut |step| {
                    let _ = progress.unbounded_send(JobEvent::Progress(step));
                },
            );
            let _ = sender.unbounded_send(JobEvent::Done(outcome));
        });
        while let Some(event) = events.next().await {
            if output.send(event).await.is_err() {
                return;
            }
        }
    })
}
