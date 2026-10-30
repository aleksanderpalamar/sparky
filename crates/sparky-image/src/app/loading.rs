use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::widget::image::Handle;
use iced::{Size, Task};
use sparky_theme::Skin;

use super::message::Message;
use super::state::{Decoding, Ready, Services, Shown, Viewer};
use crate::domain::album::{Album, Edge};
use crate::domain::picture::Picture;
use crate::domain::viewport::Viewport;
use crate::ports::{DecodeError, Decoder};

const NOTHING_OPEN: &str = "Abra uma imagem pelo gerenciador de arquivos";
const EMPTY_FOLDER: &str = "Nenhuma imagem nesta pasta";

impl Viewer {
    pub fn open(
        services: Services,
        skin: Skin,
        target: Option<PathBuf>,
        size: Size,
    ) -> (Self, Task<Message>) {
        let mut viewer = Viewer {
            album: None,
            shown: Shown::Empty(NOTHING_OPEN),
            decoding: Decoding::Idle,
            size,
            skin,
            services,
        };
        let Some(target) = target.and_then(|target| std::path::absolute(target).ok()) else {
            return (viewer, Task::none());
        };
        viewer.album = viewer.album_for(&target);
        if viewer.album.is_none() {
            viewer.shown = Shown::Empty(EMPTY_FOLDER);
            return (viewer, Task::none());
        }
        let task = viewer.load();
        (viewer, task)
    }

    pub fn load(&mut self) -> Task<Message> {
        let Some(album) = &self.album else {
            return Task::none();
        };
        let path = album.current().to_path_buf();
        self.shown = Shown::Loading;
        if self.decoding == Decoding::Busy {
            return Task::none();
        }
        self.decoding = Decoding::Busy;
        Task::perform(
            decode(Arc::clone(&self.services.decoder), path),
            |(path, result)| Message::Loaded(path, result),
        )
    }

    pub fn loaded(&mut self, path: &Path, result: Result<Picture, DecodeError>) -> Task<Message> {
        self.decoding = Decoding::Idle;
        let Some(album) = &self.album else {
            return Task::none();
        };
        if album.current() != path {
            return self.load();
        }
        self.shown = match result {
            Ok(picture) => Shown::Ready(Ready {
                viewport: Viewport::new(picture.width, picture.height),
                width: picture.width,
                height: picture.height,
                handle: Handle::from_rgba(picture.width, picture.height, picture.rgba),
            }),
            Err(error) => Shown::Failed(error.to_string()),
        };
        Task::none()
    }

    fn album_for(&self, target: &Path) -> Option<Album> {
        if !target.is_dir() {
            let folder = target.parent()?;
            return Some(Album::new(self.services.gallery.images(folder), target));
        }
        let images = self.services.gallery.images(target);
        let start = images.first()?.clone();
        let mut album = Album::new(images, &start);
        album.jump(Edge::First);
        Some(album)
    }
}

async fn decode(
    decoder: Arc<dyn Decoder>,
    path: PathBuf,
) -> (PathBuf, Result<Picture, DecodeError>) {
    let target = path.clone();
    let result = tokio::task::spawn_blocking(move || decoder.decode(&target))
        .await
        .unwrap_or_else(|error| Err(DecodeError::Unreadable(error.to_string())));
    (path, result)
}
