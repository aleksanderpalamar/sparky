use std::mem;

use crate::app::state::{Mode, Settings, Slot, Target};
use crate::domain::field::{Field, Kind};
use crate::domain::section::Section;
use crate::domain::system::Change;
use crate::domain::value::{Step, Value, parsed, read, stepped};

pub enum Opened {
    Nothing,
    Prompt,
}

impl Settings {
    pub fn count(&self) -> usize {
        match self.section {
            Section::Appearance | Section::Keyboard | Section::Apps => Field::of(self.section).len(),
            Section::Autostart => self.config.autostart.len() + 1,
            Section::Shortcuts => self.binds().len(),
            Section::Monitors => self.outputs.as_ref().map_or(0, Vec::len),
            Section::Power | Section::DateTime | Section::Region | Section::Media => self.system_rows().len(),
            Section::About => 0,
            Section::Users => self.users.len(),
        }
    }

    pub fn go(&mut self, section: Section) {
        self.section = section;
        self.focus = 0;
        self.mode = Mode::Browse;
        self.input.clear();
        self.list.follow(0, self.count());
    }

    pub fn move_focus(&mut self, delta: isize) {
        self.focus = self.focus.saturating_add_signed(delta).min(self.count().saturating_sub(1));
        self.list.follow(self.focus, self.count());
    }

    pub fn point(&mut self, index: usize) {
        if index < self.count() {
            self.focus = index;
        }
    }

    pub fn focused_field(&self) -> Option<Field> {
        Field::of(self.section).get(self.focus).copied()
    }

    pub fn nudge(&mut self, step: Step) {
        if self.is_system() {
            return self.change_system(Change::Step(step));
        }
        let Some(field) = self.focused_field() else {
            return;
        };
        if let Some(value) = stepped(field, &read(field, &self.config), step) {
            self.set(field, value);
        }
    }

    pub fn activate(&mut self) -> Opened {
        if self.is_system() {
            return self.activate_system();
        }
        if let Some(field) = self.focused_field() {
            return self.open_field(field);
        }
        if self.section != Section::Autostart {
            return Opened::Nothing;
        }
        let slot = match self.config.autostart.get(self.focus) {
            Some(_) => Slot::Existing(self.focus),
            None => Slot::New,
        };
        self.input = self.config.autostart.get(self.focus).cloned().unwrap_or_default();
        self.mode = Mode::Editing(Target::Autostart(slot));
        Opened::Prompt
    }

    pub fn remove(&mut self) {
        if self.section != Section::Autostart || self.focus >= self.config.autostart.len() {
            return;
        }
        let mut entries = self.config.autostart.clone();
        entries.remove(self.focus);
        self.set_autostart(entries);
        self.move_focus(0);
    }

    pub fn submit(&mut self) {
        let input = mem::take(&mut self.input);
        match mem::replace(&mut self.mode, Mode::Browse) {
            Mode::Editing(Target::Field(field)) => match parsed(field, &input) {
                Ok(value) => self.set(field, value),
                Err(invalid) => self.fail(invalid),
            },
            Mode::Editing(Target::Autostart(slot)) => self.save_entry(slot, input.trim()),
            Mode::Editing(Target::System(id)) => self.submit_system(id, input),
            Mode::Browse => {}
        }
    }

    pub fn escape(&mut self) {
        self.mode = Mode::Browse;
        self.input.clear();
    }

    fn open_field(&mut self, field: Field) -> Opened {
        if field.kind() == Kind::Flag {
            self.nudge(Step::More);
            return Opened::Nothing;
        }
        self.input = match read(field, &self.config) {
            Value::Number(number) => number.to_string(),
            Value::Text(text) => text,
            Value::Flag(_) => String::new(),
        };
        self.mode = Mode::Editing(Target::Field(field));
        Opened::Prompt
    }

    fn save_entry(&mut self, slot: Slot, command: &str) {
        let mut entries = self.config.autostart.clone();
        match (slot, command.is_empty()) {
            (Slot::New, true) => return,
            (Slot::New, false) => entries.push(command.to_owned()),
            (Slot::Existing(index), true) if index < entries.len() => {
                entries.remove(index);
            }
            (Slot::Existing(index), false) if index < entries.len() => entries[index] = command.to_owned(),
            (Slot::Existing(_), _) => return,
        }
        self.set_autostart(entries);
        self.move_focus(0);
    }
}
