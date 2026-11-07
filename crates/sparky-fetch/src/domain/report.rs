use serde::Serialize;

use crate::domain::field::Field;
use crate::domain::status::Status;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    pub field: Field,
    #[serde(flatten)]
    pub status: Status,
}

impl Entry {
    pub fn new(field: Field, status: Status) -> Self {
        Self { field, status }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Report {
    pub entries: Vec<Entry>,
}

impl Report {
    pub fn status(&self, field: Field) -> Option<&Status> {
        self.entries
            .iter()
            .find(|entry| entry.field == field)
            .map(|entry| &entry.status)
    }
}
