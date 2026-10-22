use iced::Size;

use super::state::Files;
use crate::domain::grid::{Direction, ViewMode};
use crate::view::style::layout_for;

impl Files {
    pub fn resize(&mut self, size: Size) {
        self.size = size;
        self.layout = layout_for(self.view_mode, size);
        self.viewport.resize(
            self.layout.rows,
            self.listing.cursor() / self.layout.columns,
            self.layout.row_count(self.listing.len()),
        );
    }

    pub fn change_view(&mut self, mode: ViewMode) {
        self.view_mode = mode;
        self.resize(self.size);
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        let index = self
            .layout
            .neighbor(self.listing.cursor(), self.listing.len(), direction);
        self.listing.select(index);
    }
}
