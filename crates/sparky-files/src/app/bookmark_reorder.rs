use iced::{Point, Task};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::drop::DropTarget;

const THRESHOLD: f32 = 8.0;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Reorder {
    #[default]
    Idle,
    Pressed {
        index: usize,
        at: Point,
    },
    Dragging {
        index: usize,
        over: Option<usize>,
    },
}

impl Files {
    pub fn press_bookmark(&mut self, index: usize) {
        self.bookmarks.reorder = Reorder::Pressed {
            index,
            at: self.pointer,
        };
    }

    pub fn pointer_at(&mut self, point: Point) -> Task<Message> {
        if let Reorder::Pressed { index, at } = self.bookmarks.reorder
            && at.distance(point) >= THRESHOLD
        {
            self.bookmarks.reorder = Reorder::Dragging { index, over: None };
        }
        self.pointer_moved(point)
    }

    pub fn hovered(&mut self, target: DropTarget) {
        if let DropTarget::Bookmark(row) = target {
            self.bookmarks.hovered = Some(row);
            if let Reorder::Dragging { over, .. } = &mut self.bookmarks.reorder {
                *over = Some(row);
            }
        }
        self.hover(target);
    }

    pub fn unhovered(&mut self, target: DropTarget) {
        if let DropTarget::Bookmark(row) = target {
            if self.bookmarks.hovered == Some(row) {
                self.bookmarks.hovered = None;
            }
            if let Reorder::Dragging { over, .. } = &mut self.bookmarks.reorder
                && *over == Some(row)
            {
                *over = None;
            }
        }
        self.unhover(target);
    }

    pub fn release_all(&mut self) -> Task<Message> {
        match std::mem::take(&mut self.bookmarks.reorder) {
            Reorder::Idle => return self.released(),
            Reorder::Pressed { index, .. } => self.open_bookmark(index),
            Reorder::Dragging {
                index,
                over: Some(to),
            } => self.move_bookmark(index, to),
            Reorder::Dragging { over: None, .. } => {}
        }
        Task::none()
    }

    pub fn cancel_reorder(&mut self) -> bool {
        let dragging = matches!(self.bookmarks.reorder, Reorder::Dragging { .. });
        self.bookmarks.reorder = Reorder::Idle;
        dragging
    }

    pub fn is_bookmark_target(&self, row: usize) -> bool {
        match self.bookmarks.reorder {
            Reorder::Dragging { index, over } => over == Some(row) && index != row,
            _ => self.is_drop_target(DropTarget::Bookmark(row)),
        }
    }

    fn move_bookmark(&mut self, from: usize, to: usize) {
        if self.bookmarks.items.reorder(from, to) && !self.save_bookmarks() {
            self.reload_bookmarks();
        }
    }
}
