use std::path::Path;

use crate::app::state::{ContextMenu, Files, Mode};
use crate::domain::bookmarks::Bookmark;
use crate::domain::context::{Action, Target};
use crate::i18n;

impl Files {
    pub fn open_bookmark_menu(&mut self, index: usize) {
        let local = self
            .bookmarks
            .items
            .get(index)
            .and_then(Bookmark::path)
            .is_some();
        if self.mode != Mode::Browse || !local {
            return;
        }
        self.cancel_reorder();
        self.context = Some(ContextMenu {
            target: Target::Bookmark(index),
            name: None,
            point: self.pointer,
        });
    }

    pub(super) fn bookmark_menu_open(&self) -> bool {
        matches!(
            self.context.as_ref().map(|menu| menu.target),
            Some(Target::Bookmark(_))
        )
    }

    pub(super) fn bookmark_action(&mut self, index: usize, action: Action) {
        let Some(path) = self
            .bookmarks
            .items
            .get(index)
            .and_then(Bookmark::path)
            .map(Path::to_path_buf)
        else {
            return;
        };
        self.reload_bookmarks();
        let Some(current) = self.bookmarks.items.find(&path) else {
            return;
        };
        match action {
            Action::Tag(color) => {
                if self.bookmarks.items.paint(current, color) {
                    self.store_or_reload();
                }
            }
            Action::RemoveBookmark => {
                let Some(removed) = self.bookmarks.items.remove(current) else {
                    return;
                };
                if self.store_or_reload() {
                    self.inform(i18n::with("bookmark-removed", "name", removed.name()));
                }
            }
            _ => {}
        }
    }

    fn store_or_reload(&mut self) -> bool {
        let saved = self.save_bookmarks();
        if !saved {
            self.reload_bookmarks();
        }
        saved
    }
}
