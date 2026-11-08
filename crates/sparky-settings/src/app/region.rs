use crate::app::state::Settings;
use crate::domain::field::Field;
use crate::domain::system::{Change, Row, repeat};
use crate::domain::value::{parsed, read, stepped};

impl Settings {
    pub fn region_rows(&self) -> Vec<Row> {
        let mut rows = self.region.rows();
        rows.extend(repeat::rows(&self.config));
        rows
    }

    pub fn change_repeat(&mut self, field: Field, change: Change) {
        match change {
            Change::Step(step) => {
                if let Some(value) = stepped(field, &read(field, &self.config), step) {
                    self.set(field, value);
                }
            }
            Change::Text(input) => match parsed(field, &input) {
                Ok(value) => self.set(field, value),
                Err(invalid) => self.fail(invalid),
            },
            Change::Activate => {}
        }
    }
}
