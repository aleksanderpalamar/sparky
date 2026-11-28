use crate::app::state::Files;
use crate::domain::history::Visit;

impl Files {
    pub fn here(&self) -> Visit {
        match self.sidebar_page {
            Some(page) => Visit::Sidebar(page),
            None => Visit::Folder(self.location.clone()),
        }
    }

    pub fn back(&mut self) {
        let Some(target) = self.history.previous().cloned() else {
            return;
        };
        let left = self.here();
        if self.arrive(target) {
            self.history.went_back(left);
        }
    }

    pub fn forward(&mut self) {
        let Some(target) = self.history.next().cloned() else {
            return;
        };
        let left = self.here();
        if self.arrive(target) {
            self.history.went_forward(left);
        }
    }

    pub(super) fn remember(&mut self, left: Visit) {
        if left != self.here() {
            self.history.visit(left);
        }
    }

    fn arrive(&mut self, visit: Visit) -> bool {
        match visit {
            Visit::Folder(path) => self.enter(path),
            Visit::Sidebar(page) => self.show_sidebar(page),
        }
    }
}
