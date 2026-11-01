use std::collections::HashMap;
use std::hash::Hash;

use iced::widget::combo_box;
use sparky_ipc::OutputInfo;

use crate::domain::screens::{mode_options, rotation_options, scale_options};
use crate::domain::system::{Control, Row, RowId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScreenField {
    Mode,
    Scale,
    Rotation,
}

#[derive(Default)]
pub struct Pickers {
    rows: HashMap<RowId, combo_box::State<String>>,
    screens: HashMap<(String, ScreenField), combo_box::State<String>>,
}

impl Pickers {
    pub fn refresh(&mut self, rows: &[Row]) {
        for row in rows {
            let Control::Choice(options) = &row.control else {
                continue;
            };
            renew(&mut self.rows, row.id, options);
        }
    }

    pub fn refresh_screens(&mut self, outputs: &[OutputInfo]) {
        for output in outputs {
            let fields = [
                (ScreenField::Mode, mode_options(output)),
                (ScreenField::Scale, scale_options(output.scale)),
                (ScreenField::Rotation, rotation_options()),
            ];
            for (field, options) in fields {
                renew(&mut self.screens, (output.name.clone(), field), &options);
            }
        }
    }

    pub fn get(&self, id: RowId) -> Option<&combo_box::State<String>> {
        self.rows.get(&id)
    }

    pub fn screen(&self, name: &str, field: ScreenField) -> Option<&combo_box::State<String>> {
        self.screens.get(&(name.to_owned(), field))
    }
}

fn renew<K: Hash + Eq>(
    states: &mut HashMap<K, combo_box::State<String>>,
    key: K,
    options: &[String],
) {
    if states.get(&key).map(combo_box::State::options) != Some(options) {
        states.insert(key, combo_box::State::new(options.to_vec()));
    }
}
