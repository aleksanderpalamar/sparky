use std::mem;
use std::path::PathBuf;
use std::time::SystemTime;

use sparky_ui::viewport::Viewport;

use crate::app::search::Search;
use crate::app::sorting::detailed;
use crate::app::state::{Files, Mode};
use crate::domain::listing::Listing;
use crate::domain::search::SearchEvent;
use crate::domain::sidebar_location::{SidebarItem, SidebarLocation};
use crate::domain::tabs::Closed;

pub struct Tab {
    pub location: PathBuf,
    pub listing: Listing,
    pub viewport: Viewport,
    pub stamp: Option<SystemTime>,
    pub sidebar_page: Option<SidebarLocation>,
    pub network_uri: Option<String>,
    pub network_pending: Option<String>,
    pub sidebar_items: Vec<SidebarItem>,
    pub sidebar_cursor: usize,
    pub search: Option<Search>,
}

impl Files {
    pub fn open_tab(&mut self) {
        let current = self.take_tab();
        let location = current.location.clone();
        let hidden = current.listing.hidden();
        let order = current.listing.order();
        self.tabs.open(current);
        self.listing = Listing::arranged(Vec::new(), hidden, order);
        self.go(location);
    }

    pub fn switch_tab(&mut self, index: usize) {
        let current = self.take_tab();
        match self.tabs.switch(index, current) {
            Ok(next) => self.load_tab(next),
            Err(current) => self.restore_tab(current),
        }
    }

    pub fn step_tab(&mut self, forward: bool) {
        let count = self.tabs.len();
        let active = self.tabs.active();
        let next = match forward {
            true => (active + 1) % count,
            false => (active + count - 1) % count,
        };
        self.switch_tab(next);
    }

    pub fn close_tab(&mut self, index: usize) {
        let closing = index == self.tabs.active();
        match self.tabs.close(index) {
            Closed::Current(next) => {
                if closing {
                    self.stop_search();
                }
                self.load_tab(next);
            }
            Closed::Other | Closed::Kept => {}
        }
    }

    pub(super) fn stored_search_event(&mut self, id: u64, event: SearchEvent) {
        let owner = |tab: &Tab| tab.search.as_ref().is_some_and(|search| search.id == id);
        let disk = self.services.disk.as_ref();
        let Some(tab) = self.tabs.find_mut(owner) else {
            return;
        };
        match event {
            SearchEvent::Found(batch) => {
                let batch = detailed(disk, &tab.location, tab.listing.order(), batch);
                tab.listing.extend(batch);
            }
            SearchEvent::Finished(finish) => {
                if let Some(search) = tab.search.as_mut() {
                    search.finish = Some(finish);
                }
            }
        }
    }

    fn take_tab(&mut self) -> Tab {
        Tab {
            location: self.location.clone(),
            listing: mem::take(&mut self.listing),
            viewport: self.viewport,
            stamp: self.stamp.take(),
            sidebar_page: self.sidebar_page.take(),
            network_uri: self.network_uri.take(),
            network_pending: self.network_pending.take(),
            sidebar_items: mem::take(&mut self.sidebar_items),
            sidebar_cursor: mem::take(&mut self.sidebar_cursor),
            search: self.search.take(),
        }
    }

    fn restore_tab(&mut self, tab: Tab) {
        self.location = tab.location;
        self.listing = tab.listing;
        self.viewport = tab.viewport;
        self.stamp = tab.stamp;
        self.sidebar_page = tab.sidebar_page;
        self.network_uri = tab.network_uri;
        self.network_pending = tab.network_pending;
        self.sidebar_items = tab.sidebar_items;
        self.sidebar_cursor = tab.sidebar_cursor;
        self.search = tab.search;
    }

    fn load_tab(&mut self, tab: Tab) {
        self.restore_tab(tab);
        self.mode = Mode::Browse;
        self.input.clear();
        self.context = None;
        self.details = None;
        self.open_with = None;
        self.refresh();
        self.resize(self.size);
    }
}
