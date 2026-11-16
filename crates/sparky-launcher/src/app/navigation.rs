use sparky_ui::viewport::Viewport;

use crate::app::state::Launcher;
use crate::app::view::{CELL_HEIGHT, ROW_STEP};
use crate::domain::grid::{Move, Window, moved, reveal, row_of, rows};
use crate::domain::shelf::{Shelf, neighbour};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scroll {
    Keep,
    To(f32),
}

impl Launcher {
    pub fn type_query(&mut self, text: String) -> Scroll {
        self.query = text;
        self.restart()
    }

    pub fn step(&mut self, step: Move) -> Scroll {
        let count = self.results().len();
        self.cursor = moved(self.cursor, count, self.mode.columns(), step);
        self.follow()
    }

    pub fn hover(&mut self, index: usize) {
        if index < self.results().len() {
            self.cursor = index;
        }
    }

    pub fn scrolled(&mut self, window: Window) {
        self.window = window;
    }

    pub fn show_shelf(&mut self, shelf: Shelf) -> Scroll {
        self.shelf = shelf;
        self.query.clear();
        self.restart()
    }

    pub fn next_shelf(&mut self, delta: isize) -> Scroll {
        let shelves = self.shelves();
        match shelves.is_empty() {
            true => Scroll::Keep,
            false => self.show_shelf(neighbour(&shelves, self.shelf, delta)),
        }
    }

    pub fn settle(&mut self) {
        self.cursor = self.cursor.min(self.results().len().saturating_sub(1));
        self.follow();
    }

    fn restart(&mut self) -> Scroll {
        self.cursor = 0;
        self.list = Viewport::new(self.mode.rows());
        self.window.offset = 0.0;
        Scroll::To(0.0)
    }

    fn follow(&mut self) -> Scroll {
        let columns = self.mode.columns();
        let row = row_of(self.cursor, columns);
        self.list.follow(row, rows(self.results().len(), columns));
        match reveal(row, ROW_STEP, CELL_HEIGHT, self.window) {
            Some(offset) if self.mode.has_shelves() => Scroll::To(offset),
            _ => Scroll::Keep,
        }
    }
}
