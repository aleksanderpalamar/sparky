use iced::mouse::ScrollDelta;
use sparky_ui::viewport::Viewport;

use crate::app::state::Launcher;
use crate::domain::grid::{Move, moved, row_of, rows};
use crate::domain::shelf::{Shelf, neighbour};

impl Launcher {
    pub fn type_query(&mut self, text: String) {
        self.query = text;
        self.restart();
    }

    pub fn step(&mut self, step: Move) {
        let count = self.results().len();
        self.cursor = moved(self.cursor, count, self.mode.columns(), step);
        self.follow();
    }

    pub fn hover(&mut self, index: usize) {
        if index < self.results().len() {
            self.cursor = index;
        }
    }

    pub fn scroll(&mut self, delta: ScrollDelta, row_height: f32) {
        let turned = self.wheel.turn(delta, row_height);
        let total = rows(self.results().len(), self.mode.columns());
        self.list.scroll(turned, total);
    }

    pub fn show_shelf(&mut self, shelf: Shelf) {
        self.shelf = shelf;
        self.query.clear();
        self.restart();
    }

    pub fn next_shelf(&mut self, delta: isize) {
        let shelves = self.shelves();
        if !shelves.is_empty() {
            self.show_shelf(neighbour(&shelves, self.shelf, delta));
        }
    }

    pub fn settle(&mut self) {
        self.cursor = self.cursor.min(self.results().len().saturating_sub(1));
        self.follow();
    }

    fn restart(&mut self) {
        self.cursor = 0;
        self.list = Viewport::new(self.mode.rows());
    }

    fn follow(&mut self) {
        let columns = self.mode.columns();
        let total = rows(self.results().len(), columns);
        self.list.follow(row_of(self.cursor, columns), total);
    }
}
