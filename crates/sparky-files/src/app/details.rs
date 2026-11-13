use crate::app::state::Files;

impl Files {
    pub fn fill_visible(&mut self) {
        if self.sidebar_page.is_some() {
            return;
        }
        let range = self.visible_range();
        for (at, name) in self.listing.pending(range) {
            let meta = self.services.disk.detail(&self.location.join(&name));
            self.listing.learn(at, meta);
        }
    }
}
