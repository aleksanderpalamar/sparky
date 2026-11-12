use std::collections::VecDeque;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::domain::entry::{Entry, Kind, Meta};
use crate::domain::listing::Hidden;
use crate::domain::search::{Finish, Query};
use crate::ports::Finder;

const BATCH: usize = 256;
const INTERVAL: Duration = Duration::from_millis(100);

pub struct TreeFinder {
    limit: usize,
}

impl TreeFinder {
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }
}

impl Finder for TreeFinder {
    fn find(
        &self,
        root: &Path,
        query: &Query,
        hidden: Hidden,
        cancel: &AtomicBool,
        found: &mut dyn FnMut(Vec<Entry>),
    ) -> Finish {
        let mut pending = VecDeque::from([PathBuf::new()]);
        let mut batch = Vec::new();
        let mut total = 0;
        let mut sent = Instant::now();
        while let Some(relative) = pending.pop_front() {
            if cancel.load(Ordering::SeqCst) {
                return Finish::Cancelled;
            }
            let Ok(items) = fs::read_dir(root.join(&relative)) else {
                continue;
            };
            for item in items.filter_map(Result::ok) {
                let name = item.file_name().to_string_lossy().into_owned();
                if hidden == Hidden::Concealed && name.starts_with('.') {
                    continue;
                }
                let path = relative.join(&name);
                let kind = kind_of(&item);
                if item.file_type().is_ok_and(|kind| kind.is_dir()) {
                    pending.push_back(path.clone());
                }
                if !query.matches(&name) {
                    continue;
                }
                batch.push(entry(&path, kind));
                total += 1;
                if total >= self.limit {
                    found(batch);
                    return Finish::Limited;
                }
                if batch.len() >= BATCH || sent.elapsed() >= INTERVAL {
                    found(std::mem::take(&mut batch));
                    sent = Instant::now();
                }
            }
        }
        if !batch.is_empty() {
            found(batch);
        }
        Finish::Complete
    }
}

fn kind_of(item: &DirEntry) -> Kind {
    let directory = match item.file_type() {
        Ok(kind) if kind.is_symlink() => fs::metadata(item.path()).is_ok_and(|meta| meta.is_dir()),
        Ok(kind) => kind.is_dir(),
        Err(_) => false,
    };
    match directory {
        true => Kind::Directory,
        false => Kind::File,
    }
}

fn entry(path: &Path, kind: Kind) -> Entry {
    Entry {
        name: path.to_string_lossy().into_owned(),
        kind,
        meta: Meta::Pending,
    }
}
