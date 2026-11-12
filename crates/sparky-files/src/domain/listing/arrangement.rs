use super::{Hidden, Listing};
use crate::domain::order::{Order, sort};

impl Listing {
    pub fn order(&self) -> Order {
        self.order
    }

    pub fn set_order(&mut self, order: Order) {
        let selected = self.selected_name();
        self.order = order;
        sort(&mut self.entries, order);
        self.refresh();
        if let Some(name) = selected {
            self.select_name(&name);
        }
    }

    pub fn hidden(&self) -> Hidden {
        self.hidden
    }

    pub fn toggle_hidden(&mut self) {
        let selected = self.selected_name();
        self.hidden = self.hidden.toggled();
        self.refresh();
        if let Some(name) = selected {
            self.select_name(&name);
        }
    }

    pub fn filter(&self) -> &str {
        &self.filter
    }

    pub fn set_filter(&mut self, filter: &str) {
        self.filter = filter.to_owned();
        self.cursor = 0;
        self.refresh();
    }
}
