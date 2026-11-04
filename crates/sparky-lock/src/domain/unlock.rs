use sparky_login_core::protocol::Refusal;
use sparky_login_look::reveal::Reveal;
use zeroize::Zeroizing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Editing,
    Waiting,
    Refused(Refusal),
}

pub struct Unlock {
    password: Zeroizing<String>,
    reveal: Reveal,
    status: Status,
}

impl Default for Unlock {
    fn default() -> Self {
        Self {
            password: Zeroizing::new(String::new()),
            reveal: Reveal::Hidden,
            status: Status::Editing,
        }
    }
}

impl Unlock {
    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn reveal(&self) -> Reveal {
        self.reveal
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn type_password(&mut self, text: String) {
        if self.status == Status::Waiting {
            return;
        }
        self.password = Zeroizing::new(text);
        self.status = Status::Editing;
    }

    pub fn toggle_reveal(&mut self) {
        self.reveal = self.reveal.toggled();
    }

    pub fn ready(&self) -> bool {
        self.status != Status::Waiting && !self.password.is_empty()
    }

    pub fn submit(&mut self) -> Option<Zeroizing<String>> {
        if !self.ready() {
            return None;
        }
        self.status = Status::Waiting;
        Some(self.password.clone())
    }

    pub fn refuse(&mut self, refusal: Refusal) {
        self.password = Zeroizing::new(String::new());
        self.status = Status::Refused(refusal);
    }

    pub fn clear(&mut self) {
        if self.status == Status::Waiting {
            return;
        }
        self.password = Zeroizing::new(String::new());
        self.status = Status::Editing;
    }

    pub fn warning(&self) -> Option<String> {
        match &self.status {
            Status::Refused(Refusal::Denied) => {
                Some(sparky_login_look::i18n::text("wrong-password"))
            }
            Status::Refused(Refusal::Unavailable(reason)) => Some(reason.clone()),
            Status::Editing | Status::Waiting => None,
        }
    }
}
