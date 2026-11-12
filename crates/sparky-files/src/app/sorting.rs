use std::io;
use std::path::Path;

use fluent_bundle::FluentArgs;

use crate::app::state::Files;
use crate::domain::entry::Entry;
use crate::domain::order::{Order, SortDirection, SortKey};
use crate::i18n;

impl Files {
    pub fn arrange(&mut self, order: Order) {
        if self.sidebar_page.is_some() {
            return;
        }
        if order.needs_details() {
            for (at, name) in self.listing.unknown() {
                let meta = self.services.disk.detail(&self.location.join(&name));
                self.listing.learn(at, meta);
            }
        }
        self.listing.set_order(order);
        self.follow();
        self.inform(described(order));
    }

    pub(super) fn listed(&self, path: &Path) -> io::Result<Vec<Entry>> {
        let mut entries = self.services.disk.list(path)?;
        if self.listing.order().needs_details() {
            for entry in &mut entries {
                entry.meta = self.services.disk.detail(&path.join(&entry.name));
            }
        }
        Ok(entries)
    }
}

pub fn described(order: Order) -> String {
    let key = match order.key {
        SortKey::Name => "sort-name",
        SortKey::Size => "sort-size",
        SortKey::Modified => "sort-modified",
        SortKey::Type => "sort-type",
    };
    let mut args = FluentArgs::new();
    args.set("key", i18n::text(key));
    args.set("arrow", arrow(order.direction));
    i18n::message("sorted-by", Some(&args))
}

pub fn arrow(direction: SortDirection) -> &'static str {
    match direction {
        SortDirection::Ascending => "↑",
        SortDirection::Descending => "↓",
    }
}
