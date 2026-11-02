use crate::domain::power::Power;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Username,
    Password,
    Login,
    Power(Power),
    Session,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Focus::Username => Focus::Password,
            Focus::Password => Focus::Login,
            Focus::Login => Focus::Power(Power::Shutdown),
            Focus::Power(Power::Shutdown) => Focus::Power(Power::Reboot),
            Focus::Power(Power::Reboot) => Focus::Power(Power::Suspend),
            Focus::Power(Power::Suspend) => Focus::Session,
            Focus::Session => Focus::Username,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Focus::Username => Focus::Session,
            Focus::Password => Focus::Username,
            Focus::Login => Focus::Password,
            Focus::Power(Power::Shutdown) => Focus::Login,
            Focus::Power(Power::Reboot) => Focus::Power(Power::Shutdown),
            Focus::Power(Power::Suspend) => Focus::Power(Power::Reboot),
            Focus::Session => Focus::Power(Power::Suspend),
        }
    }

    pub fn is_field(self) -> bool {
        matches!(self, Focus::Username | Focus::Password)
    }
}
