use std::mem;

use iced::{Point, Task};

use crate::app::drop::Gesture;
use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::thumbnail_key::file_uri;
use crate::i18n;

impl Files {
    pub(super) fn leave_window(&mut self, point: Point) -> Task<Message> {
        if self.inside_window(point) || !matches!(self.gesture, Gesture::Dragging { .. }) {
            return Task::none();
        }
        let Gesture::Dragging { names, .. } = mem::replace(&mut self.gesture, Gesture::Idle) else {
            return Task::none();
        };
        let uris = names
            .iter()
            .map(|name| file_uri(&self.location.join(name)))
            .collect();
        let dragger = self.services.dragger.clone();
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || dragger.start(uris))
                    .await
                    .map_err(|error| error.to_string())
                    .and_then(|result| result)
            },
            |result| match result {
                Ok(()) => Message::Released,
                Err(reason) => Message::DragRefused(reason),
            },
        )
    }

    pub fn drag_refused(&mut self, reason: &str) {
        self.fail(i18n::with("drag-refused", "reason", reason.to_owned()));
    }

    fn inside_window(&self, point: Point) -> bool {
        (0.0..self.size.width).contains(&point.x) && (0.0..self.size.height).contains(&point.y)
    }
}
