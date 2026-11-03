use zeroize::Zeroizing;

use crate::domain::focus::Focus;

const DENIED: &str = "Falha no login!";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reveal {
    Hidden,
    Shown,
}

pub use sparky_login_core::protocol::Refusal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Editing,
    Waiting,
    Refused(Refusal),
}

pub struct Credentials {
    pub username: String,
    pub password: Zeroizing<String>,
}

pub struct Form {
    username: String,
    password: Zeroizing<String>,
    reveal: Reveal,
    status: Status,
}

impl Form {
    pub fn new(last_user: Option<&str>) -> Self {
        Self {
            username: last_user.map(str::to_lowercase).unwrap_or_default(),
            password: Zeroizing::new(String::new()),
            reveal: Reveal::Hidden,
            status: Status::Editing,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn reveal(&self) -> Reveal {
        self.reveal
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn first_focus(&self) -> Focus {
        match self.username.is_empty() {
            true => Focus::Username,
            false => Focus::Password,
        }
    }

    pub fn type_username(&mut self, text: String) {
        if self.edit() {
            self.username = text.to_lowercase();
        }
    }

    pub fn type_password(&mut self, text: String) {
        if self.edit() {
            self.password = Zeroizing::new(text);
        }
    }

    pub fn toggle_reveal(&mut self) {
        self.reveal = match self.reveal {
            Reveal::Hidden => Reveal::Shown,
            Reveal::Shown => Reveal::Hidden,
        };
    }

    pub fn ready(&self) -> bool {
        self.status != Status::Waiting && !self.username.is_empty() && !self.password.is_empty()
    }

    pub fn submit(&mut self) -> Option<Credentials> {
        if !self.ready() {
            return None;
        }
        self.status = Status::Waiting;
        Some(Credentials {
            username: self.username.clone(),
            password: self.password.clone(),
        })
    }

    pub fn refuse(&mut self, refusal: Refusal) {
        self.password = Zeroizing::new(String::new());
        self.status = Status::Refused(refusal);
    }

    pub fn warning(&self) -> Option<String> {
        match &self.status {
            Status::Refused(Refusal::Denied) => Some(DENIED.to_owned()),
            Status::Refused(Refusal::Unavailable(reason)) => Some(reason.clone()),
            Status::Editing | Status::Waiting => None,
        }
    }

    fn edit(&mut self) -> bool {
        if self.status == Status::Waiting {
            return false;
        }
        self.status = Status::Editing;
        true
    }
}
