#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tabs<T> {
    slots: Vec<Option<T>>,
    active: usize,
}

impl<T> Default for Tabs<T> {
    fn default() -> Self {
        Self {
            slots: vec![None],
            active: 0,
        }
    }
}

impl<T> Tabs<T> {
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn active(&self) -> usize {
        self.active
    }

    pub fn open(&mut self, current: T) {
        self.store(current);
        self.active += 1;
        self.slots.insert(self.active, None);
    }

    pub fn switch(&mut self, index: usize, current: T) -> Result<T, T> {
        if index == self.active || index >= self.slots.len() {
            return Err(current);
        }
        let Some(next) = self.slots.get_mut(index).and_then(Option::take) else {
            return Err(current);
        };
        self.store(current);
        self.active = index;
        Ok(next)
    }

    pub fn close(&mut self, index: usize) -> Closed<T> {
        if self.slots.len() < 2 || index >= self.slots.len() {
            return Closed::Kept;
        }
        if index != self.active {
            self.slots.remove(index);
            if index < self.active {
                self.active -= 1;
            }
            return Closed::Other;
        }
        self.slots.remove(index);
        self.active = index.min(self.slots.len() - 1);
        match self.slots.get_mut(self.active).and_then(Option::take) {
            Some(next) => Closed::Current(next),
            None => Closed::Kept,
        }
    }

    pub fn stored(&self, index: usize) -> Option<&T> {
        self.slots.get(index).and_then(Option::as_ref)
    }

    pub fn find_mut(&mut self, wanted: impl Fn(&T) -> bool) -> Option<&mut T> {
        self.slots.iter_mut().flatten().find(|tab| wanted(tab))
    }

    fn store(&mut self, current: T) {
        if let Some(slot) = self.slots.get_mut(self.active) {
            *slot = Some(current);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Closed<T> {
    Current(T),
    Other,
    Kept,
}
