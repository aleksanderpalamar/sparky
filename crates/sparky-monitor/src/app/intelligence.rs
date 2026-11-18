use std::sync::Arc;

use iced::Task;

use super::message::Message;
use super::page::Page;
use super::state::{Monitor, Sampling};
use crate::domain::intelligence::Advice;

impl Monitor {
    pub fn advise(&mut self) -> Task<Message> {
        if self.page != Page::Intelligence || self.advising == Sampling::Busy {
            return Task::none();
        }
        self.advising = Sampling::Busy;
        let advisor = Arc::clone(&self.services.advisor);
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || advisor.advise())
                    .await
                    .ok()
            },
            |advice| Message::Advised(Box::new(advice.unwrap_or(Advice::Unavailable))),
        )
    }

    pub fn advised(&mut self, advice: Advice) {
        self.advising = Sampling::Idle;
        self.advice = Some(advice);
    }
}
