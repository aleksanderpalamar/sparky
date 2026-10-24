#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabAction {
    New,
    Close,
    Cycle(i8),
}

pub struct Tab<T> {
    pub id: TabId,
    pub value: T,
}

pub struct Tabs<T> {
    entries: Vec<Tab<T>>,
    active: Option<TabId>,
    next: Option<u64>,
}

impl<T> Tabs<T> {
    pub fn new(first: T) -> Self {
        Self {
            entries: vec![Tab {
                id: TabId(0),
                value: first,
            }],
            active: Some(TabId(0)),
            next: Some(1),
        }
    }

    pub fn insert(&mut self, value: T) -> Option<TabId> {
        let next = self.next?;
        self.next = next.checked_add(1);
        let id = TabId(next);
        self.entries.push(Tab { id, value });
        self.active = Some(id);
        Some(id)
    }

    pub fn active_id(&self) -> Option<TabId> {
        self.active
    }

    pub fn active(&self) -> Option<&Tab<T>> {
        self.get(self.active?)
    }

    pub fn active_mut(&mut self) -> Option<&mut Tab<T>> {
        self.get_mut(self.active?)
    }

    pub fn get(&self, id: TabId) -> Option<&Tab<T>> {
        self.entries.iter().find(|tab| tab.id == id)
    }

    pub fn get_mut(&mut self, id: TabId) -> Option<&mut Tab<T>> {
        self.entries.iter_mut().find(|tab| tab.id == id)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Tab<T>> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Tab<T>> {
        self.entries.iter_mut()
    }

    pub fn select(&mut self, id: TabId) -> bool {
        if self.get(id).is_none() {
            return false;
        }
        self.active = Some(id);
        true
    }

    pub fn cycle(&mut self, step: i8) {
        let Some(index) = self
            .entries
            .iter()
            .position(|tab| Some(tab.id) == self.active)
        else {
            return;
        };
        let index =
            (index as i128 + i128::from(step)).rem_euclid(self.entries.len() as i128) as usize;
        self.active = Some(self.entries[index].id);
    }

    pub fn remove(&mut self, id: TabId) -> Option<T> {
        let index = self.entries.iter().position(|tab| tab.id == id)?;
        let removed = self.entries.remove(index);
        if self.active != Some(id) {
            return Some(removed.value);
        }
        self.active = self
            .entries
            .get(index)
            .or_else(|| self.entries.last())
            .map(|tab| tab.id);
        Some(removed.value)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
