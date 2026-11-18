use crate::domain::{Mode, ServiceState, Switch};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Knowledge {
    #[default]
    Unvalidated,
    Validated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Link {
    #[default]
    Available,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Budget {
    #[default]
    Within,
    Exceeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    RestoreUnconfirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Ready,
    Turned(Switch),
    Chose(Mode),
    Learned(Knowledge),
    Context(Link),
    Spending(Budget),
    Failed(Fault),
    Cleared,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Lifecycle {
    ready: bool,
    switch: Switch,
    mode: Mode,
    knowledge: Knowledge,
    context: Link,
    budget: Budget,
    fault: Option<Fault>,
}

impl Lifecycle {
    pub fn receive(&mut self, signal: Signal) {
        match signal {
            Signal::Ready => self.ready = true,
            Signal::Turned(switch) => self.switch = switch,
            Signal::Chose(mode) => self.mode = mode,
            Signal::Learned(knowledge) => self.knowledge = knowledge,
            Signal::Context(link) => self.context = link,
            Signal::Spending(budget) => self.budget = budget,
            Signal::Failed(fault) => self.fault = Some(fault),
            Signal::Cleared => self.fault = None,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn switch(&self) -> Switch {
        self.switch
    }

    pub fn state(&self) -> ServiceState {
        if self.switch == Switch::Off {
            return ServiceState::Disabled;
        }
        if !self.ready {
            return ServiceState::Starting;
        }
        if self.fault.is_some() {
            return ServiceState::Safe;
        }
        if self.budget == Budget::Exceeded {
            return ServiceState::Degraded;
        }
        self.chosen_state()
    }

    fn chosen_state(&self) -> ServiceState {
        if self.mode == Mode::Observation || self.knowledge == Knowledge::Unvalidated {
            return ServiceState::Observing;
        }
        if self.mode == Mode::Conservative || self.context == Link::Lost {
            return ServiceState::Conservative;
        }
        ServiceState::Adaptive
    }
}
