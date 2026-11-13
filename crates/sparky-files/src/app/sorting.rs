use std::io;
use std::path::Path;

use fluent_bundle::FluentArgs;

use crate::app::state::Files;
use crate::domain::entry::Entry;
use crate::domain::order::{Order, SortDirection, SortKey};
use crate::i18n;
use crate::ports::Disk;

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
        let entries = self.services.disk.list(path)?;
        let disk = self.services.disk.as_ref();
        Ok(detailed(disk, path, self.listing.order(), entries))
    }
}

pub(super) fn detailed(
    disk: &dyn Disk,
    dir: &Path,
    order: Order,
    mut entries: Vec<Entry>,
) -> Vec<Entry> {
    if order.needs_details() {
        for entry in &mut entries {
            entry.meta = disk.detail(&dir.join(&entry.name));
        }
    }
    entries
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
