use sparky_config::Config;

use crate::domain::field::{Field, Kind};
use crate::domain::system::{Control, Row, RowId};
use crate::domain::value::{read, shown};

const ROWS: [(RowId, Field); 2] = [
    (RowId::RepeatRate, Field::RepeatRate),
    (RowId::RepeatDelay, Field::RepeatDelay),
];

pub fn rows(config: &Config) -> Vec<Row> {
    ROWS.into_iter()
        .filter_map(|(id, field)| {
            let Kind::Number(bounds) = field.kind() else {
                return None;
            };
            Some(Row {
                id,
                label: field.label(),
                value: shown(&read(field, config)),
                control: Control::Number(bounds),
            })
        })
        .collect()
}

pub fn field(id: RowId) -> Option<Field> {
    ROWS.into_iter()
        .find(|(row, _)| *row == id)
        .map(|(_, field)| field)
}
