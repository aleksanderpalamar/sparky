use std::time::Duration;

use sparky_ipc::SessionAction;

pub const CLOSE_GRACE: Duration = Duration::from_secs(8);
pub const LOCK_GRACE: Duration = Duration::from_secs(5);
pub const TICK: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Farewell {
    Logout,
    Reboot,
    Poweroff,
}

impl Farewell {
    pub fn power(self) -> Option<&'static str> {
        match self {
            Farewell::Logout => None,
            Farewell::Reboot => Some("reboot"),
            Farewell::Poweroff => Some("poweroff"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    Lock,
    Suspend,
    Leave(Farewell),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Closing {
    Waiting,
    Done,
    Stuck(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sleep {
    Waiting,
    Now,
}

pub fn plan(action: SessionAction) -> Plan {
    match action {
        SessionAction::Lock => Plan::Lock,
        SessionAction::Suspend => Plan::Suspend,
        SessionAction::Logout => Plan::Leave(Farewell::Logout),
        SessionAction::Reboot => Plan::Leave(Farewell::Reboot),
        SessionAction::Poweroff => Plan::Leave(Farewell::Poweroff),
    }
}

pub fn allowed(plan: Plan, real_session: bool) -> bool {
    real_session || matches!(plan, Plan::Lock | Plan::Leave(Farewell::Logout))
}

pub fn closing(windows: usize, elapsed: Duration) -> Closing {
    match (windows, elapsed >= CLOSE_GRACE) {
        (0, _) => Closing::Done,
        (left, true) => Closing::Stuck(left),
        (_, false) => Closing::Waiting,
    }
}

pub fn sleep(locked: bool, elapsed: Duration) -> Sleep {
    match locked || elapsed >= LOCK_GRACE {
        true => Sleep::Now,
        false => Sleep::Waiting,
    }
}

pub fn stuck_message(left: usize) -> String {
    let windows = match left {
        1 => "1 janela aberta".to_owned(),
        many => format!("{many} janelas abertas"),
    };
    format!("Ainda há {windows}. Salve e feche, ou saia à força com SUPER + SHIFT + ESC.")
}
