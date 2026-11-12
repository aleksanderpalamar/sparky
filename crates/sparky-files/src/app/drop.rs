use std::mem;
use std::path::PathBuf;

use iced::keyboard::Modifiers;
use iced::{Point, Task};

use crate::app::jobs::Origin;
use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::domain::clip::Clip;
use crate::domain::drop::{DropTarget, Override, action};
use crate::domain::removal::Removal;
use crate::domain::transfer::Action;
use crate::i18n;

const THRESHOLD: f32 = 8.0;

#[derive(Debug, Clone, PartialEq)]
pub enum Gesture {
    Idle,
    Pressed {
        index: usize,
        at: Point,
    },
    Dragging {
        names: Vec<String>,
        over: Option<DropTarget>,
    },
}

impl Files {
    pub fn pressed(&mut self, index: usize) {
        let allowed = self.mode == Mode::Browse && self.sidebar_page.is_none() && !self.in_trash();
        self.gesture = match allowed {
            true => Gesture::Pressed {
                index,
                at: self.pointer,
            },
            false => Gesture::Idle,
        };
    }

    pub fn pointer_moved(&mut self, point: Point) {
        self.pointer = point;
        let Gesture::Pressed { index, at } = self.gesture else {
            return;
        };
        if at.distance(point) < THRESHOLD {
            return;
        }
        let names = self.dragged(index);
        self.gesture = match names.is_empty() {
            true => Gesture::Idle,
            false => Gesture::Dragging { names, over: None },
        };
    }

    pub fn hover(&mut self, target: DropTarget) {
        if let Gesture::Dragging { over, .. } = &mut self.gesture {
            *over = Some(target);
        }
    }

    pub fn unhover(&mut self, target: DropTarget) {
        if let Gesture::Dragging { over, .. } = &mut self.gesture
            && *over == Some(target)
        {
            *over = None;
        }
    }

    pub fn hold_keys(&mut self, modifiers: Modifiers) {
        self.held = Override::from_keys(modifiers.control(), modifiers.shift());
    }

    pub fn cancel_drag(&mut self) -> bool {
        let dragging = matches!(self.gesture, Gesture::Dragging { .. });
        self.gesture = Gesture::Idle;
        dragging
    }

    pub fn is_drop_target(&self, target: DropTarget) -> bool {
        matches!(&self.gesture, Gesture::Dragging { over: Some(over), .. } if *over == target)
    }

    pub fn drag_status(&self) -> Option<String> {
        let Gesture::Dragging { names, .. } = &self.gesture else {
            return None;
        };
        Some(i18n::counted("drag-hint", names.len()))
    }

    pub fn released(&mut self) -> Task<Message> {
        let gesture = mem::replace(&mut self.gesture, Gesture::Idle);
        let Gesture::Dragging {
            names,
            over: Some(target),
        } = gesture
        else {
            return Task::none();
        };
        if target == DropTarget::Trash {
            if self.services.disk.has_trash(&self.location) {
                self.mode = Mode::ConfirmRemoval(Removal::Trash, names);
            }
            return Task::none();
        }
        let Some(into) = self.drop_folder(&names, target) else {
            return Task::none();
        };
        let paths: Vec<PathBuf> = names.iter().map(|name| self.location.join(name)).collect();
        if paths
            .iter()
            .all(|path| path.parent() == Some(into.as_path()))
        {
            return Task::none();
        }
        let disk = self.services.disk.as_ref();
        let source = disk.device(&self.location);
        let same = source.is_some() && source == disk.device(&into);
        let clip = match action(self.held, same) {
            Action::Copy => Clip::Copy(paths),
            Action::Move => Clip::Cut(paths),
        };
        self.transfer_from(clip, into, Origin::Dropped)
    }

    fn drop_folder(&self, names: &[String], target: DropTarget) -> Option<PathBuf> {
        match target {
            DropTarget::Entry(index) => {
                let entry = self.listing.get(index)?;
                (entry.is_directory() && !names.contains(&entry.name))
                    .then(|| self.location.join(&entry.name))
            }
            DropTarget::Place(index) => self.places.get(index).map(|place| place.path.clone()),
            DropTarget::Tab(index) => self
                .tabs
                .stored(index)
                .filter(|tab| tab.sidebar_page.is_none() && tab.search.is_none())
                .map(|tab| tab.location.clone()),
            DropTarget::Trash => None,
        }
    }

    fn dragged(&self, index: usize) -> Vec<String> {
        let Some(entry) = self.listing.get(index) else {
            return Vec::new();
        };
        match self.listing.is_marked(entry) {
            true => self.listing.chosen(),
            false => vec![entry.name.clone()],
        }
    }
}
