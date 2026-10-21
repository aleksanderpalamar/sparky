use crate::domain::item::{Attributes, Edit, Item, NewItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Existing {
    Keep,
    Replace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stored {
    Created(u64),
    Replaced(u64),
}

impl Stored {
    pub fn id(self) -> u64 {
        match self {
            Stored::Created(id) | Stored::Replaced(id) => id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vault {
    pub created: u64,
    pub modified: u64,
    items: Vec<Item>,
}

impl Vault {
    pub fn new(now: u64) -> Self {
        Self { created: now, modified: now, items: Vec::new() }
    }

    pub fn restore(created: u64, modified: u64, items: Vec<Item>) -> Self {
        Self { created, modified, items }
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn item(&self, id: u64) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn search(&self, query: &Attributes) -> Vec<u64> {
        self.items.iter().filter(|item| item.matches(query)).map(|item| item.id).collect()
    }

    pub fn store(&mut self, new: NewItem, existing: Existing, now: u64) -> Stored {
        self.modified = now;
        let same = match existing {
            Existing::Replace => self.items.iter_mut().find(|item| item.attributes == new.attributes),
            Existing::Keep => None,
        };
        if let Some(item) = same {
            item.label = new.label;
            item.secret = new.secret;
            item.modified = now;
            return Stored::Replaced(item.id);
        }
        let id = self.next_id();
        self.items.push(Item { id, label: new.label, attributes: new.attributes, secret: new.secret, created: now, modified: now });
        Stored::Created(id)
    }

    pub fn delete(&mut self, id: u64, now: u64) -> bool {
        let before = self.items.len();
        self.items.retain(|item| item.id != id);
        let deleted = self.items.len() < before;
        if deleted {
            self.modified = now;
        }
        deleted
    }

    pub fn edit(&mut self, id: u64, edit: Edit, now: u64) -> bool {
        let Some(item) = self.items.iter_mut().find(|item| item.id == id) else {
            return false;
        };
        match edit {
            Edit::Label(label) => item.label = label,
            Edit::Attributes(attributes) => item.attributes = attributes,
            Edit::Secret(secret) => item.secret = secret,
        }
        item.modified = now;
        self.modified = now;
        true
    }

    fn next_id(&self) -> u64 {
        self.items.iter().map(|item| item.id).max().map_or(1, |id| id + 1)
    }
}
