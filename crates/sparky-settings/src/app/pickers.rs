use std::collections::HashMap;

use iced::widget::combo_box;

use crate::domain::system::{Control, Row, RowId};

#[derive(Default)]
pub struct Pickers(HashMap<RowId, combo_box::State<String>>);

impl Pickers {
    pub fn refresh(&mut self, rows: &[Row]) {
        for row in rows {
            let Control::Choice(options) = &row.control else {
                continue;
            };
            let current = self.0.get(&row.id).map(combo_box::State::options);
            if current != Some(options.as_slice()) {
                self.0
                    .insert(row.id, combo_box::State::new(options.clone()));
            }
        }
    }

    pub fn get(&self, id: RowId) -> Option<&combo_box::State<String>> {
        self.0.get(&id)
    }
}
