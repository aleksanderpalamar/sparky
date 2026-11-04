use crate::app::state::Settings;
use crate::domain::system::idle::{self, KEY, LABEL, TABLE};
use crate::domain::system::{Change, Row};
use crate::domain::value::Step;

impl Settings {
    pub fn power_rows(&self) -> Vec<Row> {
        let mut rows = self.power.rows();
        rows.push(idle::row(self.config.lock.idle_minutes));
        rows
    }

    pub fn change_idle(&mut self, change: Change) {
        let current = self.config.lock.idle_minutes;
        let minutes = match change {
            Change::Step(step) => Some(idle::stepped(current, step)),
            Change::Activate => Some(idle::stepped(current, Step::More)),
            Change::Text(text) => idle::chosen(&text),
        };
        let Some(minutes) = minutes else {
            return self.fail(format!("{LABEL}: escolha uma das opções"));
        };
        let Ok(number) = i32::try_from(minutes) else {
            return;
        };
        let what = format!("{LABEL}: {}", idle::label(minutes));
        self.edit(&what, |editor| editor.set_number(TABLE, KEY, number));
    }
}
