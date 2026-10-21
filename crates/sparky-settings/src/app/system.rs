use crate::app::state::Settings;
use crate::domain::section::Section;
use crate::domain::system::power::Power;
use crate::domain::system::{Change, Request, Row, perform};

impl Settings {
    pub fn is_system(&self) -> bool {
        matches!(self.section, Section::Power)
    }

    pub fn system_rows(&self) -> Vec<Row> {
        match self.section {
            Section::Power => self.power.rows(),
            _ => Vec::new(),
        }
    }

    pub fn reload_system(&mut self) {
        self.power = Power::load(self.services.commands.as_ref());
    }

    pub fn change_system(&mut self, change: Change) {
        let Some(row) = self.system_rows().into_iter().nth(self.focus) else {
            return;
        };
        let Some(request) = self.system_request(&row, change) else {
            return;
        };
        match perform(self.services.commands.as_ref(), &request) {
            Ok(()) => {
                self.reload_system();
                let value = self.system_rows().into_iter().find(|fresh| fresh.id == row.id).map(|fresh| fresh.value);
                self.inform(format!("{}: {}", row.label, value.unwrap_or_default()));
            }
            Err(error) => self.fail(format!("{}: {error}", row.label)),
        }
    }

    fn system_request(&self, row: &Row, change: Change) -> Option<Request> {
        match self.section {
            Section::Power => self.power.request(row.id, change),
            _ => None,
        }
    }
}
