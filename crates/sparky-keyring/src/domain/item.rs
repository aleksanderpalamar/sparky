use std::collections::BTreeMap;

use crate::domain::secret::Secret;

pub type Attributes = BTreeMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: u64,
    pub label: String,
    pub attributes: Attributes,
    pub secret: Secret,
    pub created: u64,
    pub modified: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewItem {
    pub label: String,
    pub attributes: Attributes,
    pub secret: Secret,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Label(String),
    Attributes(Attributes),
    Secret(Secret),
}

impl Item {
    pub fn matches(&self, query: &Attributes) -> bool {
        query.iter().all(|(name, value)| self.attributes.get(name) == Some(value))
    }
}
