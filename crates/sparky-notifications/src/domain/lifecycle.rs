use super::{Center, Error};

impl Center {
    pub fn invoke(&mut self, id: u32, key: &str) -> Result<Option<(u32, u32)>, Error> {
        let (_, resident) = self.action(id, key).ok_or(Error::Unknown)?;
        if resident {
            return Ok(None);
        }
        Ok(self.dismiss(id, 2))
    }

    pub fn expire(&mut self, now: u64) -> Vec<(u32, u32)> {
        let expired: Vec<_> = self
            .active
            .iter()
            .filter_map(|(id, active)| {
                active
                    .deadline
                    .filter(|deadline| *deadline <= now)
                    .map(|_| *id)
            })
            .collect();
        expired
            .into_iter()
            .filter_map(|id| self.dismiss(id, 1))
            .collect()
    }

    pub fn clear(&mut self) -> Vec<(u32, u32)> {
        let closed = self.active.keys().map(|id| (*id, 2)).collect();
        if self.active.is_empty() && self.history.is_empty() {
            return closed;
        }
        self.active.clear();
        self.history.clear();
        self.changed();
        closed
    }

    pub fn set_dnd(&mut self, enabled: bool) {
        if self.dnd == enabled {
            return;
        }
        self.dnd = enabled;
        if enabled {
            self.hide_popups();
        }
        self.changed();
    }

    pub fn center_open(&mut self, owner: &str, open: bool) {
        let changed = match open {
            true => self.readers.insert(owner.to_owned()),
            false => self.readers.remove(owner),
        };
        if !changed {
            return;
        }
        if open {
            self.hide_popups();
        }
        self.changed();
    }

    pub fn owner_lost(&mut self, owner: &str) -> Vec<(u32, u32)> {
        self.center_open(owner, false);
        let mut ids = Vec::new();
        for (id, active) in &mut self.active {
            if active.sender != owner {
                continue;
            }
            active.sender.clear();
            active.card.notification.actions.clear();
            ids.push(*id);
        }
        if ids.is_empty() {
            return Vec::new();
        }
        for card in &mut self.history {
            if ids.contains(&card.id) {
                card.notification.actions.clear();
            }
        }
        self.changed();
        Vec::new()
    }

    fn hide_popups(&mut self) {
        for active in self.active.values_mut() {
            active.popup = false;
        }
    }
}
