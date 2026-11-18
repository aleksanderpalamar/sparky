use crate::application::{Insight, Knowledge, Lifecycle, Link, Signal, training};
use crate::domain::{Activities, ServiceState, Switch, TrainingState};
use crate::protocol::{Refusal, Reply, Request, Status};

#[derive(Debug, Default)]
pub struct Service {
    lifecycle: Lifecycle,
    learning: Switch,
    activities: Activities,
    insights: Vec<Insight>,
    training_active: bool,
    forget_requested: bool,
}

impl Service {
    pub fn ready(&mut self) {
        self.lifecycle.receive(Signal::Ready);
    }

    pub fn observe(&mut self, activities: Activities, link: Link, insights: Vec<Insight>) {
        self.activities = activities;
        self.insights = insights;
        self.lifecycle.receive(Signal::Context(link));
    }

    pub fn learned(&mut self, knowledge: Knowledge) {
        self.lifecycle.receive(Signal::Learned(knowledge));
    }

    pub fn set_training_active(&mut self, active: bool) {
        self.training_active = active;
    }

    pub fn take_forget(&mut self) -> bool {
        std::mem::take(&mut self.forget_requested)
    }

    pub fn training(&self) -> TrainingState {
        match training(self.learning, self.state(), self.activities) {
            TrainingState::Allowed if self.training_active => TrainingState::Active,
            other => other,
        }
    }

    pub fn state(&self) -> ServiceState {
        self.lifecycle.state()
    }

    pub fn answer(&mut self, request: Request) -> Reply {
        match request {
            Request::Status => Reply::Status(self.status()),
            Request::Insight => Reply::Insight {
                metrics: self.insights.clone(),
            },
            Request::SetMode { mode } => self.apply(Signal::Chose(mode)),
            Request::SetEnabled { switch } => self.apply(Signal::Turned(switch)),
            Request::SetLearning { switch } => {
                self.learning = switch;
                Reply::Done
            }
            Request::ClearSafeMode => self.clear_safe_mode(),
            Request::ForgetKnowledge => {
                self.forget_requested = true;
                Reply::Done
            }
        }
    }

    fn apply(&mut self, signal: Signal) -> Reply {
        self.lifecycle.receive(signal);
        Reply::Done
    }

    fn clear_safe_mode(&mut self) -> Reply {
        if self.state() != ServiceState::Safe {
            return Reply::Refused {
                reason: Refusal::NotInSafeMode,
            };
        }
        self.apply(Signal::Cleared)
    }

    fn status(&self) -> Status {
        let state = self.state();
        Status {
            service: env!("CARGO_PKG_VERSION").to_owned(),
            state,
            enabled: self.lifecycle.switch(),
            mode: self.lifecycle.mode(),
            learning: self.learning,
            training: self.training(),
            activities: self.activities,
        }
    }
}
