use std::sync::{Arc, Mutex};

use crate::domain::choice::{Outcome, Request};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shown {
    Chosen(Outcome),
    Cancelled,
    Failed,
}

pub trait Answer {
    fn send(&self, outcome: &Outcome);
}

pub trait Dialogs: Send + Sync {
    fn show(&self, request: &Request, closer: &Closer) -> Shown;
}

type Action = Box<dyn FnOnce() + Send>;

#[derive(Default)]
struct Closing {
    closed: bool,
    action: Option<Action>,
}

#[derive(Clone, Default)]
pub struct Closer(Arc<Mutex<Closing>>);

impl Closer {
    pub fn on_close(&self, action: impl FnOnce() + Send + 'static) {
        let Ok(mut closing) = self.0.lock() else {
            return;
        };
        match closing.closed {
            true => {
                drop(closing);
                action();
            }
            false => closing.action = Some(Box::new(action)),
        }
    }

    pub fn close(&self) {
        let action = match self.0.lock() {
            Ok(mut closing) => {
                closing.closed = true;
                closing.action.take()
            }
            Err(_) => None,
        };
        if let Some(action) = action {
            action();
        }
    }

    pub fn closed(&self) -> bool {
        self.0.lock().map(|closing| closing.closed).unwrap_or(true)
    }
}
