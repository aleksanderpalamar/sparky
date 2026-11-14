use std::path::{Path, PathBuf};

use fluent_bundle::FluentArgs;

use crate::app::bookmark_reorder::Reorder;
use crate::app::state::Files;
use crate::domain::bookmarks::{Added, Bookmarks};
use crate::domain::entry::Kind;
use crate::i18n;

#[derive(Debug, Clone, Default)]
pub struct BookmarkBar {
    pub items: Bookmarks,
    pub hovered: Option<usize>,
    pub(super) reorder: Reorder,
}

impl Files {
    pub fn reload_bookmarks(&mut self) {
        if self.bookmarks.reorder != Reorder::Idle || self.bookmark_menu_open() {
            return;
        }
        if let Ok(items) = self.services.bookmarks.load() {
            self.bookmarks.items = Bookmarks::new(items);
        }
    }

    pub fn bookmark_location(&mut self) {
        if self.sidebar_page.is_some() || self.in_trash() {
            self.inform(i18n::text("bookmark-unavailable"));
            return;
        }
        self.add_bookmarks(vec![self.location.clone()]);
    }

    pub(super) fn bookmark_dropped(&mut self, names: Vec<String>) {
        let disk = self.services.disk.as_ref();
        let folders: Vec<PathBuf> = names
            .iter()
            .map(|name| self.location.join(name))
            .filter(|path| {
                disk.properties(path)
                    .is_ok_and(|item| item.kind == Kind::Directory)
            })
            .collect();
        match folders.is_empty() {
            true => self.inform(i18n::text("bookmark-only-folders")),
            false => self.add_bookmarks(folders),
        }
    }

    pub(super) fn open_bookmark(&mut self, index: usize) {
        let Some(bookmark) = self.bookmarks.items.get(index) else {
            return;
        };
        let name = bookmark.name();
        let Some(path) = bookmark.path().map(Path::to_path_buf) else {
            return;
        };
        if !self.services.disk.exists(&path) {
            self.fail(i18n::with("bookmark-missing", "name", name));
            return;
        }
        self.go(path);
    }

    pub(super) fn save_bookmarks(&mut self) -> bool {
        match self.services.bookmarks.save(self.bookmarks.items.items()) {
            Ok(()) => true,
            Err(error) => {
                self.fail(i18n::with("bookmark-failed", "error", error));
                false
            }
        }
    }

    fn add_bookmarks(&mut self, paths: Vec<PathBuf>) {
        self.reload_bookmarks();
        let first = paths.first().map(|path| shown(path));
        let added = paths
            .into_iter()
            .filter(|path| self.bookmarks.items.add(path.clone()) == Added::New)
            .count();
        if added > 0 && !self.save_bookmarks() {
            return;
        }
        let text = match (added, first) {
            (0, Some(name)) => i18n::with("bookmark-exists", "name", name),
            (1, Some(name)) => i18n::with("bookmark-added-one", "name", name),
            (count, _) => {
                let mut args = FluentArgs::new();
                args.set("count", count);
                i18n::message("bookmark-added-many", Some(&args))
            }
        };
        self.inform(text);
    }
}

fn shown(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}
