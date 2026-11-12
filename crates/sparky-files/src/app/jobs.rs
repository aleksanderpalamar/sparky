use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use iced::Task;
use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};

use crate::app::extraction::ExtractReport;
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
    pub work: Work,
    pub origin: Origin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Work {
    Transfer(Action),
    Extract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    User,
    Dropped,
    Undo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    pub clip: Clip,
    pub into: PathBuf,
    pub names: Vec<String>,
    pub origin: Origin,
}

#[derive(Debug, Clone)]
pub enum JobEvent {
    Progress(Progress),
    Done(Outcome),
    Extracted(ExtractReport),
}

impl Files {
    pub fn paste(&mut self) -> Task<Message> {
        let Some(clip) = self.clipboard_clip() else {
            self.inform(i18n::text("nothing-to-paste"));
            return Task::none();
        };
        self.transfer(clip, self.location.clone())
    }

    pub fn transfer(&mut self, clip: Clip, into: PathBuf) -> Task<Message> {
        self.transfer_from(clip, into, Origin::User)
    }

    pub fn transfer_from(&mut self, clip: Clip, into: PathBuf, origin: Origin) -> Task<Message> {
        if self.job.is_some() {
            self.fail(i18n::text("job-busy"));
            return Task::none();
        }
        let names = operations::conflicts(&clip, &into, self.services.disk.as_ref());
        if !names.is_empty() {
            self.mode = Mode::Conflict(Pending {
                clip,
                into,
                names,
                origin,
            });
            return Task::none();
        }
        self.start_transfer(clip, into, Conflict::KeepBoth, origin)
    }

    pub fn resolve_conflict(&mut self, choice: Conflict) -> Task<Message> {
        let Mode::Conflict(pending) = std::mem::replace(&mut self.mode, Mode::Browse) else {
            return Task::none();
        };
        self.start_transfer(pending.clip, pending.into, choice, pending.origin)
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
                if let Some(Job {
                    work: Work::Transfer(action),
                    origin,
                    ..
                }) = self.job.take()
                {
                    self.transferred(action, origin, outcome);
                }
            }
            JobEvent::Extracted(report) => {
                if self.job.take().is_some() {
                    self.extracted(report);
                }
            }
        }
    }

    fn start_transfer(
        &mut self,
        clip: Clip,
        into: PathBuf,
        conflict: Conflict,
        origin: Origin,
    ) -> Task<Message> {
        let planned = match operations::plan(&clip, &into, self.services.disk.as_ref(), conflict) {
            Ok(planned) => planned,
            Err(failure) => {
                self.fail(failure);
                return Task::none();
            }
        };
        let action = match clip {
            Clip::Copy(_) => Action::Copy,
            Clip::Cut(_) => Action::Move,
        };
        self.run_job(planned, action, origin)
    }

    pub(super) fn run_job(
        &mut self,
        planned: Planned,
        action: Action,
        origin: Origin,
    ) -> Task<Message> {
        let cancel = Arc::new(AtomicBool::new(false));
        self.job = Some(Job {
            cancel: cancel.clone(),
            progress: Progress::default(),
            work: Work::Transfer(action),
            origin,
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
    background(move |send| {
        let outcome = execute(
            planned.steps,
            planned.skipped,
            transfer.as_ref(),
            &cancel,
            &mut |step| send(JobEvent::Progress(step)),
        );
        send(JobEvent::Done(outcome));
    })
}

pub(super) fn background<T: Send + 'static>(
    work: impl FnOnce(&dyn Fn(T)) + Send + 'static,
) -> impl Stream<Item = T> {
    iced::stream::channel(16, async move |mut output| {
        let (sender, mut events) = mpsc::unbounded();
        thread::spawn(move || {
            work(&|event| {
                let _ = sender.unbounded_send(event);
            })
        });
        while let Some(event) = events.next().await {
            if output.send(event).await.is_err() {
                return;
            }
        }
    })
}
