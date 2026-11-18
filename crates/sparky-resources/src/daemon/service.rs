use crate::application::{Budget, Fault, Insight, Knowledge, Lifecycle, Link, Signal, training};
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
    decisions: Vec<crate::orchestration::Decision>,
    protected: Vec<String>,
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

    pub fn configure(&mut self, settings: &sparky_config::Resources) {
        let mode = match settings.mode {
            sparky_config::ResourceMode::Adaptive => crate::domain::Mode::Adaptive,
            sparky_config::ResourceMode::Observation => crate::domain::Mode::Observation,
            sparky_config::ResourceMode::Conservative => crate::domain::Mode::Conservative,
        };
        self.lifecycle.receive(Signal::Chose(mode));
        self.lifecycle.receive(Signal::Turned(if settings.enabled {
            Switch::On
        } else {
            Switch::Off
        }));
        self.learning = if settings.learning {
            Switch::On
        } else {
            Switch::Off
        };
        self.protected = names(&settings.protected).unwrap_or_default();
    }

    pub fn protected(&self) -> &[String] {
        &self.protected
    }

    pub fn decided(&mut self, decisions: Vec<crate::orchestration::Decision>) {
        self.decisions = decisions;
    }

    pub fn spending(&mut self, budget: Budget) {
        self.lifecycle.receive(Signal::Spending(budget));
    }

    pub fn failed(&mut self, fault: Fault) {
        self.lifecycle.receive(Signal::Failed(fault));
    }

    pub fn activities(&self) -> crate::domain::Activities {
        self.activities
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
            Request::Decisions => Reply::Decisions {
                entries: self.decisions.clone(),
            },
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
            Request::SetProtected { names: list } => match names(&list) {
                Some(valid) => {
                    self.protected = valid;
                    Reply::Done
                }
                None => Reply::Refused {
                    reason: Refusal::InvalidNames,
                },
            },
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

const MAX_NAMES: usize = 64;
const MAX_NAME: usize = 64;

fn names(list: &[String]) -> Option<Vec<String>> {
    let valid = list.len() <= MAX_NAMES
        && list
            .iter()
            .all(|name| !name.trim().is_empty() && name.len() <= MAX_NAME && !name.contains('\0'));
    valid.then(|| list.iter().map(|name| name.trim().to_owned()).collect())
}
