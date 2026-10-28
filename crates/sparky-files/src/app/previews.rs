use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use iced::Task;
use iced::widget::image::Handle;

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::entry::Kind;
use crate::domain::grid::ViewMode;
use crate::domain::thumbnail::{Thumbnail, supported};

const CACHE_LIMIT: usize = 64;
const JOB_LIMIT: usize = 2;

struct Cached {
    stamp: Option<SystemTime>,
    handle: Option<Handle>,
}

#[derive(Default)]
pub struct Previews {
    cache: HashMap<PathBuf, Cached>,
    order: VecDeque<PathBuf>,
    pending: HashMap<PathBuf, Option<SystemTime>>,
    visible: usize,
}

impl Previews {
    pub fn handle(&self, path: &Path) -> Option<Handle> {
        self.cache
            .get(path)
            .and_then(|cached| cached.handle.clone())
    }

    fn seen(&self, path: &Path, stamp: Option<SystemTime>) -> bool {
        self.pending.contains_key(path)
            || self
                .cache
                .get(path)
                .is_some_and(|cached| cached.stamp == stamp)
    }

    fn begin(&mut self, path: PathBuf, stamp: Option<SystemTime>) {
        self.pending.insert(path, stamp);
    }

    fn finish(&mut self, path: PathBuf, stamp: Option<SystemTime>, image: Option<Thumbnail>) {
        if self.pending.remove(&path) != Some(stamp) {
            return;
        }
        let handle = image.map(|image| Handle::from_rgba(image.width, image.height, image.rgba));
        self.order.retain(|item| item != &path);
        self.order.push_back(path.clone());
        self.cache.insert(path, Cached { stamp, handle });
        while self.cache.len() > CACHE_LIMIT.max(self.visible) {
            if let Some(oldest) = self.order.pop_front() {
                self.cache.remove(&oldest);
            }
        }
    }
}

impl Files {
    pub fn request_previews(&mut self) -> Task<Message> {
        let range = match self.view_mode {
            ViewMode::Grid => self.layout.range(self.viewport.top(), self.listing.len()),
            ViewMode::List => self.viewport.range(self.listing.len()),
        };
        self.previews.visible = range.len();
        let candidates: Vec<PathBuf> = range
            .filter_map(|index| self.listing.get(index))
            .filter(|entry| entry.kind == Kind::File)
            .map(|entry| self.location.join(&entry.name))
            .filter(|path| supported(path))
            .collect();
        let mut tasks = Vec::new();
        for path in candidates {
            if self.previews.pending.len() >= JOB_LIMIT {
                break;
            }
            let stamp = self.services.disk.stamp(&path);
            if self.previews.seen(&path, stamp) {
                continue;
            }
            self.previews.begin(path.clone(), stamp);
            let source = self.services.thumbnails.clone();
            let source_path = path.clone();
            tasks.push(Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || source.load(&source_path))
                        .await
                        .ok()
                        .flatten()
                },
                move |image| Message::PreviewReady(path, stamp, image),
            ));
        }
        Task::batch(tasks)
    }

    pub fn preview_ready(
        &mut self,
        path: PathBuf,
        stamp: Option<SystemTime>,
        image: Option<Thumbnail>,
    ) {
        if self.services.disk.stamp(&path) != stamp {
            self.previews.pending.remove(&path);
            return;
        }
        self.previews.finish(path, stamp, image);
    }
}

#[cfg(test)]
mod tests {
    use super::{CACHE_LIMIT, Previews, Thumbnail};
    use std::path::PathBuf;

    #[test]
    fn cache_e_limitado_e_renova_arquivo_modificado() {
        let mut previews = Previews::default();
        for index in 0..=CACHE_LIMIT {
            let path = PathBuf::from(format!("{index}.png"));
            previews.begin(path.clone(), None);
            previews.finish(path, None, None);
        }
        assert_eq!(previews.cache.len(), CACHE_LIMIT);
        assert!(!previews.cache.contains_key(&PathBuf::from("0.png")));
        let path = PathBuf::from("64.png");
        assert!(previews.seen(&path, None));
        let stamp = Some(std::time::SystemTime::now());
        assert!(!previews.seen(&path, stamp));
        previews.begin(path.clone(), stamp);
        previews.finish(
            path.clone(),
            stamp,
            Some(Thumbnail {
                width: 1,
                height: 1,
                rgba: vec![0; 4],
            }),
        );
        assert!(previews.handle(&path).is_some());
    }
}
