use crate::domain::Change;
use crate::i18n::{message, text};
use crate::orchestration::{Decision, Purpose, Verdict};
use crate::safety::Outcome;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub title: String,
    pub body: String,
}

pub fn notice(decision: &Decision) -> Option<Notice> {
    let key = match decision.verdict {
        Verdict::Applied => "notice-applied",
        Verdict::Reverted(_) => "notice-reverted",
        Verdict::Finished(Outcome::Unconfirmed) => "notice-unconfirmed",
        _ => return None,
    };
    let values = [
        ("change", text(change(&decision.change))),
        ("process", decision.process.clone()),
        ("detail", decision.change.summary()),
        ("purpose", text(purpose(decision.purpose))),
    ];
    Some(Notice {
        title: text(&format!("{key}-title")),
        body: message(key, &values),
    })
}

fn change(change: &Change) -> &'static str {
    match change {
        Change::Nice(_) => "change-nice",
        Change::Affinity(_) => "change-affinity",
        Change::IoPriority(_) => "change-io",
    }
}

fn purpose(purpose: Purpose) -> &'static str {
    match purpose {
        Purpose::EaseCpu => "purpose-ease-cpu",
        Purpose::EaseIo => "purpose-ease-io",
    }
}
