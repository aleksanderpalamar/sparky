use std::path::{Path, PathBuf};

use crate::app::context;
use crate::app::state::Files;
use crate::domain::listing::Hidden;

impl Files {
    pub fn reveal(&mut self, items: &[PathBuf]) {
        let Some(parent) = items.first().and_then(|item| item.parent()) else {
            return;
        };
        if !self.go(parent.to_path_buf()) {
            return;
        }
        let names: Vec<String> = items
            .iter()
            .filter(|item| item.parent() == Some(parent))
            .filter_map(|item| item.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .collect();
        let hidden = names.iter().any(|name| name.starts_with('.'));
        if hidden && self.listing.hidden() == Hidden::Concealed {
            self.listing.toggle_hidden();
        }
        if names.len() > 1 {
            for name in &names {
                self.listing.select_name(name);
                self.listing.toggle_mark();
            }
        }
        if let Some(first) = names.first() {
            self.listing.select_name(first);
        }
        self.follow();
    }

    pub fn reveal_properties(&mut self, item: &Path) {
        self.reveal(&[item.to_path_buf()]);
        context::properties(self, item);
    }
}
