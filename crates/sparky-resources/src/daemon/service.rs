use crate::application::{Insight, Lifecycle, Link, Signal, training};
use crate::domain::{Activities, ServiceState, Switch};
use crate::protocol::{Refusal, Reply, Request, Status};

#[derive(Debug, Default)]
pub struct Service {
    lifecycle: Lifecycle,
    learning: Switch,
    activities: Activities,
    insights: Vec<Insight>,
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
            training: training(self.learning, state, self.activities),
            activities: self.activities,
        }
    }
}
