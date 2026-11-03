use crate::domain::system::{Control, Row, RowId};
use crate::domain::value::Step;

pub const TABLE: &str = "lock";
pub const KEY: &str = "idle_minutes";
pub const LABEL: &str = "Bloquear por inatividade";
const CHOICES: [u32; 8] = [0, 1, 2, 5, 10, 15, 30, 60];
const HOUR: u32 = 60;

pub fn row(minutes: u32) -> Row {
    Row {
        id: RowId::IdleLock,
        label: LABEL.to_owned(),
        value: label(minutes),
        control: Control::Choice(CHOICES.into_iter().map(label).collect()),
    }
}

pub fn label(minutes: u32) -> String {
    match minutes {
        0 => "Nunca".to_owned(),
        1 => "1 minuto".to_owned(),
        HOUR => "1 hora".to_owned(),
        many => format!("{many} minutos"),
    }
}

pub fn chosen(text: &str) -> Option<u32> {
    CHOICES
        .into_iter()
        .find(|minutes| label(*minutes) == text.trim())
}

pub fn stepped(minutes: u32, step: Step) -> u32 {
    let neighbor = match step {
        Step::More => CHOICES.into_iter().find(|choice| *choice > minutes),
        Step::Less => CHOICES.into_iter().rev().find(|choice| *choice < minutes),
    };
    neighbor.unwrap_or(minutes.min(HOUR))
}
