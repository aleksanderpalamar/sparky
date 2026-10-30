use super::message::{Move, Selection};
use super::page::Page;
use super::state::{Focus, Monitor};
use crate::domain::table::{Column, Order, Sort};

impl Monitor {
    pub fn open(&mut self, page: Page) {
        self.page = page;
        self.confirm = None;
        if let Some(column) = page.resource() {
            self.sort = Sort {
                column,
                order: Order::Descending,
            };
            self.resort();
        }
    }

    pub fn sort_by(&mut self, column: Column) {
        self.sort = self.sort.by(column);
        self.resort();
    }

    pub fn switch_focus(&mut self) {
        if self.page != Page::Dashboard {
            return;
        }
        self.focus = match self.focus {
            Focus::Applications => Focus::Processes,
            Focus::Processes => Focus::Applications,
        };
    }

    pub fn select(&mut self, selection: Selection) {
        match selection {
            Selection::Process(pid) => {
                self.selected_process = Some(pid);
                self.focus = Focus::Processes;
            }
            Selection::App(id) => {
                self.selected_app = Some(id);
                self.focus = Focus::Applications;
            }
        }
    }

    pub fn step(&mut self, movement: Move) {
        match self.active_table() {
            Some(Focus::Processes) => {
                let pids: Vec<u32> = self.processes.iter().map(|row| row.pid).collect();
                self.selected_process = moved(&pids, self.selected_process.as_ref(), movement);
            }
            Some(Focus::Applications) => {
                let ids: Vec<String> = self.apps.iter().map(|app| app.id.clone()).collect();
                self.selected_app = moved(&ids, self.selected_app.as_ref(), movement);
            }
            None => {}
        }
    }

    pub fn scrolled(&mut self, lines: isize) {
        let movement = match lines.is_negative() {
            true => Move::Up,
            false => Move::Down,
        };
        for _ in 0..lines.unsigned_abs() {
            self.step(movement);
        }
    }
}

fn moved<T: Clone + PartialEq>(keys: &[T], current: Option<&T>, movement: Move) -> Option<T> {
    let last = keys.len().checked_sub(1)?;
    let index = current.and_then(|current| keys.iter().position(|key| key == current));
    let target = match (movement, index) {
        (Move::First, _) | (Move::Up | Move::Down, None) => 0,
        (Move::Last, _) => last,
        (Move::Down, Some(index)) => (index + 1).min(last),
        (Move::Up, Some(index)) => index.saturating_sub(1),
    };
    keys.get(target).cloned()
}
