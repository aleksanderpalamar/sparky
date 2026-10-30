use iced::Task;

use super::message::Message;
use super::state::{Shown, Viewer};
use crate::domain::album::{Album, Edge, Step};
use crate::domain::viewport::{Area, Direction, Viewport};

impl Viewer {
    pub fn step(&mut self, step: Step) -> Task<Message> {
        self.move_to(|album| album.step(step))
    }

    pub fn jump(&mut self, edge: Edge) -> Task<Message> {
        self.move_to(|album| album.jump(edge))
    }

    pub fn arrow(&mut self, direction: Direction) -> Task<Message> {
        let area = self.area();
        if let Shown::Ready(ready) = &mut self.shown
            && ready.viewport.overflows(area)
        {
            ready.viewport.look(direction, area);
            return Task::none();
        }
        match direction {
            Direction::Left => self.step(Step::Previous),
            Direction::Right => self.step(Step::Next),
            Direction::Up | Direction::Down => Task::none(),
        }
    }

    pub fn adjust(&mut self, change: impl FnOnce(&mut Viewport, Area)) {
        let area = self.area();
        if let Shown::Ready(ready) = &mut self.shown {
            change(&mut ready.viewport, area);
        }
    }

    fn move_to(&mut self, change: impl FnOnce(&mut Album)) -> Task<Message> {
        let Some(album) = &mut self.album else {
            return Task::none();
        };
        let before = album.position();
        change(album);
        if album.position() == before {
            return Task::none();
        }
        self.load()
    }
}
