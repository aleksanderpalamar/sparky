use crate::app::actions::Opened;
use crate::app::state::{Mode, Settings, Target};
use crate::domain::section::Section;
use crate::domain::system::datetime::DateTime;
use crate::domain::system::power::Power;
use crate::domain::system::region::Region;
use crate::domain::system::users;
use crate::domain::system::media::Media;
use crate::domain::system::themes::Themes;
use crate::domain::system::{Change, Control, Request, Row, RowId, perform};

impl Settings {
    pub fn is_system(&self) -> bool {
        matches!(self.section, Section::Power | Section::DateTime | Section::Region | Section::Media | Section::Themes)
    }

    pub fn system_rows(&self) -> Vec<Row> {
        match self.section {
            Section::Power => self.power.rows(),
            Section::DateTime => self.datetime.rows(),
            Section::Region => self.region.rows(),
            Section::Media => self.media.rows(),
            Section::Themes => self.theme_rows(),
            _ => Vec::new(),
        }
    }

    pub fn reload_system(&mut self) {
        self.power = Power::load(self.services.commands.as_ref());
        self.datetime = DateTime::load(self.services.commands.as_ref());
        self.region = Region::load(self.services.commands.as_ref());
        self.users = users::load(self.services.commands.as_ref());
        self.media = Media::load(self.services.commands.as_ref());
        self.themes = Themes::load(self.services.themes.as_ref());
        self.move_focus(0);
    }

    pub fn activate_system(&mut self) -> Opened {
        let Some(row) = self.system_rows().into_iter().nth(self.focus) else {
            return Opened::Nothing;
        };
        if row.control == Control::Text {
            self.input = row.value;
            self.mode = Mode::Editing(Target::System(row.id));
            return Opened::Prompt;
        }
        self.change_system(Change::Activate);
        Opened::Nothing
    }

    pub fn change_system(&mut self, change: Change) {
        if self.section == Section::Themes {
            return self.change_theme(change);
        }
        let Some(row) = self.system_rows().into_iter().nth(self.focus) else {
            return;
        };
        self.apply_system(&row, change);
    }

    pub fn submit_system(&mut self, id: RowId, input: String) {
        let Some(row) = self.system_rows().into_iter().find(|row| row.id == id) else {
            return;
        };
        self.apply_system(&row, Change::Text(input));
    }

    fn apply_system(&mut self, row: &Row, change: Change) {
        let Some(request) = self.system_request(row, change) else {
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

    fn system_request(&mut self, row: &Row, change: Change) -> Option<Request> {
        match (self.section, row.id, change) {
            (Section::Power, id, change) => self.power.request(id, change),
            (Section::DateTime, RowId::Timezone, Change::Text(input)) => self.timezone_request(input),
            (Section::DateTime, id, change) => self.datetime.request(id, change),
            (Section::Region, id, change) => self.region_request(id, change),
            (Section::Media, id, change) => self.media.request(id, change),
            _ => None,
        }
    }

    fn timezone_request(&mut self, input: String) -> Option<Request> {
        if let Some(unknown) = self.datetime.unknown_timezone(&input) {
            self.fail(format!("fuso desconhecido: {unknown}"));
            return None;
        }
        self.datetime.request(RowId::Timezone, Change::Text(input))
    }

    fn region_request(&mut self, id: RowId, change: Change) -> Option<Request> {
        let invalid = match &change {
            Change::Text(input) => self.region.unknown_value(id, input),
            _ => None,
        };
        if let Some(unknown) = invalid {
            self.fail(format!("valor indisponível: {unknown}"));
            return None;
        }
        self.region.request(id, change)
    }
}
