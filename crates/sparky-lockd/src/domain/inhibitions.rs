#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hold {
    Free,
    Held,
}

#[derive(Debug, Default)]
pub struct Inhibitions {
    last: u32,
    active: Vec<(u32, String)>,
}

impl Inhibitions {
    pub fn hold(&self) -> Hold {
        match self.active.is_empty() {
            true => Hold::Free,
            false => Hold::Held,
        }
    }

    pub fn inhibit(&mut self, owner: &str) -> (u32, Option<Hold>) {
        let before = self.hold();
        let cookie = self.next_cookie();
        self.active.push((cookie, owner.to_owned()));
        (cookie, self.changed(before))
    }

    pub fn release(&mut self, cookie: u32) -> Option<Hold> {
        let before = self.hold();
        self.active.retain(|(active, _)| *active != cookie);
        self.changed(before)
    }

    pub fn forget(&mut self, owner: &str) -> Option<Hold> {
        let before = self.hold();
        self.active.retain(|(_, active)| active != owner);
        self.changed(before)
    }

    fn next_cookie(&mut self) -> u32 {
        self.last = self.last.checked_add(1).unwrap_or(1);
        self.last
    }

    fn changed(&self, before: Hold) -> Option<Hold> {
        let now = self.hold();
        (now != before).then_some(now)
    }
}
