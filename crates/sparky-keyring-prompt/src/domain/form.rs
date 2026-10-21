use std::fmt;

use zeroize::Zeroizing;

use crate::domain::mode::Mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Password,
    Confirmation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Problem {
    Empty,
    Mismatch,
}

impl Problem {
    pub fn message(self) -> &'static str {
        match self {
            Problem::Empty => "A senha não pode ficar vazia.",
            Problem::Mismatch => "As senhas não conferem.",
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum Step {
    Focus(Field),
    Done(Zeroizing<String>),
    Stay,
}

impl fmt::Debug for Step {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Step::Focus(field) => write!(formatter, "Focus({field:?})"),
            Step::Done(_) => formatter.write_str("Done(..)"),
            Step::Stay => formatter.write_str("Stay"),
        }
    }
}

pub struct Form {
    pub mode: Mode,
    pub problem: Option<Problem>,
    password: Zeroizing<String>,
    confirmation: Zeroizing<String>,
    focused: Field,
}

impl Form {
    pub fn new(mode: Mode) -> Self {
        Self { mode, problem: None, password: Zeroizing::default(), confirmation: Zeroizing::default(), focused: Field::Password }
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn confirmation(&self) -> &str {
        &self.confirmation
    }

    pub fn type_in(&mut self, field: Field, text: String) {
        let target = match field {
            Field::Password => &mut self.password,
            Field::Confirmation => &mut self.confirmation,
        };
        *target = Zeroizing::new(text);
        self.focused = field;
        self.problem = None;
    }

    pub fn switch(&mut self) -> Option<Field> {
        if !self.mode.confirms() {
            return None;
        }
        self.focused = match self.focused {
            Field::Password => Field::Confirmation,
            Field::Confirmation => Field::Password,
        };
        Some(self.focused)
    }

    pub fn enter(&mut self, field: Field) -> Step {
        self.focused = field;
        if self.password.is_empty() {
            self.problem = Some(Problem::Empty);
            return self.focus(Field::Password, field);
        }
        match (self.mode.confirms(), field) {
            (false, _) => Step::Done(self.password.clone()),
            (true, Field::Password) => self.focus(Field::Confirmation, field),
            (true, Field::Confirmation) => self.confirm(),
        }
    }

    fn confirm(&mut self) -> Step {
        if *self.confirmation != *self.password {
            self.confirmation = Zeroizing::default();
            self.problem = Some(Problem::Mismatch);
            return Step::Stay;
        }
        Step::Done(self.password.clone())
    }

    fn focus(&mut self, wanted: Field, current: Field) -> Step {
        if wanted == current {
            return Step::Stay;
        }
        self.focused = wanted;
        Step::Focus(wanted)
    }
}
