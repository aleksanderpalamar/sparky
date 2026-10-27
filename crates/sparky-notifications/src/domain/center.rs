use super::{Card, Error, Incoming, Moment, Notification, Snapshot, model::Active, sanitize};
use std::collections::{BTreeMap, VecDeque};

#[derive(Default)]
pub struct Center {
    pub(super) active: BTreeMap<u32, Active>,
    pub(super) history: VecDeque<Card>,
    pub(super) next: u32,
    pub(super) revision: u64,
    pub(super) dnd: bool,
    pub(super) readers: std::collections::HashSet<String>,
}

impl Center {
    pub fn notify(&mut self, input: Incoming, now: impl Into<Moment>) -> Result<u32, Error> {
        let now = now.into();
        let id = self.id(&input)?;
        let timeout = sanitize::timeout(input.timeout, input.urgency);
        let card = Card {
            id,
            created_ms: now.utc_ms,
            active: true,
            notification: Notification {
                app: sanitize::text(&input.app, 128),
                icon: sanitize::text(&input.icon, 1024),
                picture: input.picture.filter(sanitize::picture),
                title: sanitize::text(&input.title, 200),
                body: sanitize::text(&input.body, 4000),
                actions: input
                    .actions
                    .into_iter()
                    .take(8)
                    .filter(|action| {
                        !action.key.is_empty()
                            && action.key.len() <= 128
                            && action.label.len() <= 256
                    })
                    .collect(),
                urgency: input.urgency,
            },
        };
        self.history.retain(|entry| entry.id != id);
        if !input.transient {
            self.history.push_front(card.clone());
        }
        self.history.truncate(100);
        self.active.insert(
            id,
            Active {
                sender: input.sender,
                card,
                deadline: timeout.map(|value| now.elapsed_ms.saturating_add(value)),
                popup: !self.dnd && self.readers.is_empty(),
                resident: input.resident,
            },
        );
        self.limit_popups();
        self.changed();
        Ok(id)
    }

    pub fn snapshot_revision(&self) -> u64 {
        self.revision
    }

    pub fn snapshot(&self) -> Snapshot {
        let items = self
            .history
            .iter()
            .map(|card| {
                let mut card = card.clone();
                card.active = self.active.contains_key(&card.id);
                if !card.active {
                    card.notification.actions.clear();
                }
                card
            })
            .collect();
        Snapshot {
            revision: self.revision,
            dnd: self.dnd,
            items,
        }
    }

    pub fn popups(&self) -> Vec<Card> {
        self.active
            .values()
            .filter(|active| active.popup)
            .map(|active| active.card.clone())
            .collect()
    }

    pub fn close_client(&mut self, id: u32, sender: &str) -> Result<(u32, u32), Error> {
        let active = self.active.get(&id).ok_or(Error::Unknown)?;
        if active.sender != sender {
            return Err(Error::Sender);
        }
        self.dismiss(id, 3).ok_or(Error::Unknown)
    }

    pub fn dismiss(&mut self, id: u32, reason: u32) -> Option<(u32, u32)> {
        let active = self.active.remove(&id);
        let previous = self.history.len();
        if reason != 1 {
            self.history.retain(|card| card.id != id);
        }
        if active.is_some() || previous != self.history.len() {
            self.changed();
        }
        active.map(|_| (id, reason))
    }

    pub fn action(&self, id: u32, key: &str) -> Option<(String, bool)> {
        let active = self.active.get(&id)?;
        active
            .card
            .notification
            .actions
            .iter()
            .find(|action| action.key == key)?;
        Some((active.sender.clone(), active.resident))
    }

    pub fn next_deadline(&self) -> Option<u64> {
        self.active
            .values()
            .filter_map(|entry| entry.deadline)
            .min()
    }

    pub(super) fn changed(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }

    fn id(&mut self, input: &Incoming) -> Result<u32, Error> {
        if input.replaces != 0 {
            let active = self.active.get(&input.replaces).ok_or(Error::Unknown)?;
            return match active.sender == input.sender {
                true => Ok(input.replaces),
                false => Err(Error::Sender),
            };
        }
        if self.active.len() >= 128 {
            return Err(Error::Full);
        }
        loop {
            self.next = self.next.wrapping_add(1);
            if self.next != 0 && !self.active.contains_key(&self.next) {
                return Ok(self.next);
            }
        }
    }

    fn limit_popups(&mut self) {
        let mut visible: Vec<_> = self
            .active
            .iter()
            .filter(|(_, active)| active.popup)
            .map(|(id, active)| (active.card.created_ms, *id))
            .collect();
        visible.sort_unstable_by(|a, b| b.cmp(a));
        for (_, id) in visible.into_iter().skip(3) {
            if let Some(active) = self.active.get_mut(&id) {
                active.popup = false;
            }
        }
    }
}
