use std::sync::Arc;
use std::time::Instant;

use iced::Task;

use super::message::Message;
use super::state::{Monitor, Sampling};
use crate::domain::apps::group;
use crate::domain::measure::measure;
use crate::domain::snapshot::Snapshot;
use crate::domain::table::sort;

impl Monitor {
    pub fn tick(&mut self) -> Task<Message> {
        if self.sampling == Sampling::Busy {
            return Task::none();
        }
        self.sampling = Sampling::Busy;
        let sampler = Arc::clone(&self.services.sampler);
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || sampler.sample(Instant::now()))
                    .await
                    .ok()
            },
            |snapshot| match snapshot {
                Some(snapshot) => Message::Sampled(Box::new(snapshot)),
                None => Message::Tick,
            },
        )
    }

    pub fn load_catalog(&self) -> Task<Message> {
        let catalog = Arc::clone(&self.services.catalog);
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || catalog.apps())
                    .await
                    .unwrap_or_default()
            },
            Message::Catalog,
        )
    }

    pub fn sampled(&mut self, snapshot: Snapshot) {
        self.sampling = Sampling::Idle;
        let mut reading = measure(self.previous.as_ref().unwrap_or(&snapshot), &snapshot);
        self.histories.record(&reading);
        self.apps = group(&reading.processes, &self.catalog, self.services.user);
        self.processes = std::mem::take(&mut reading.processes);
        self.reading = Some(reading);
        self.previous = Some(snapshot);
        self.resort();
        self.forget_vanished();
    }

    pub fn resort(&mut self) {
        sort(&mut self.processes, self.sort);
        sort(&mut self.apps, self.sort);
    }

    fn forget_vanished(&mut self) {
        if let Some(pid) = self.selected_process
            && !self.processes.iter().any(|row| row.pid == pid)
        {
            self.selected_process = None;
        }
        if let Some(id) = &self.selected_app
            && !self.apps.iter().any(|app| &app.id == id)
        {
            self.selected_app = None;
        }
    }
}
