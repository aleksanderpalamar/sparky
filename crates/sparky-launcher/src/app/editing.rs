use crate::app::state::Launcher;
use crate::domain::menu::{Action, Item, Target, items};
use crate::domain::overlay::{Menu, Overlay, Purpose};
use crate::domain::shelf::{Shelf, after_removal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Search,
    Name,
    Stay,
}

impl Launcher {
    pub fn open_menu(&mut self, target: Target) {
        if !self.mode.has_shelves() || items(&target, &self.groups, self.shelf).is_empty() {
            return;
        }
        self.overlay = Overlay::Menu(Menu {
            target,
            point: self.pointer,
            cursor: 0,
        });
    }

    pub fn open_menu_on_chosen(&mut self) {
        if let Some(app) = self.chosen() {
            self.open_menu(Target::App(app.id));
        }
    }

    pub fn menu_items(&self) -> Vec<Item> {
        match &self.overlay {
            Overlay::Menu(menu) => items(&menu.target, &self.groups, self.shelf),
            _ => Vec::new(),
        }
    }

    pub fn menu_step(&mut self, delta: isize) {
        let last = self.menu_items().len().saturating_sub(1);
        if let Overlay::Menu(menu) = &mut self.overlay {
            menu.cursor = menu.cursor.saturating_add_signed(delta).min(last);
        }
    }

    pub fn choose(&mut self, index: usize) -> Focus {
        let Some(item) = self.menu_items().into_iter().nth(index) else {
            return Focus::Stay;
        };
        self.overlay = Overlay::Closed;
        match item.action {
            Action::AddTo { group, app } => {
                self.edited(|launcher| launcher.groups.add(group, &app))
            }
            Action::TakeFrom { group, app } => {
                self.edited(|launcher| launcher.groups.take(group, &app))
            }
            Action::NewGroupWith(app) => self.ask_name(Purpose::Create { with: Some(app) }, ""),
            Action::Rename(group) => {
                let name = self.groups.get(group).map(|group| group.name.clone());
                self.ask_name(Purpose::Rename(group), &name.unwrap_or_default())
            }
            Action::Remove(group) => self.edited(|launcher| launcher.remove_group(group)),
        }
    }

    pub fn add_group(&mut self) -> Focus {
        self.ask_name(Purpose::Create { with: None }, "")
    }

    pub fn type_name(&mut self, typed: String) {
        if let Overlay::Naming { text, refused, .. } = &mut self.overlay {
            *text = typed;
            *refused = false;
        }
    }

    pub fn confirm_name(&mut self) -> Focus {
        let Overlay::Naming { purpose, text, .. } = self.overlay.clone() else {
            return Focus::Stay;
        };
        let done = match purpose {
            Purpose::Create { with } => self.create_group(&text, with),
            Purpose::Rename(group) => self.groups.rename(group, &text),
        };
        if let (false, Overlay::Naming { refused, .. }) = (done, &mut self.overlay) {
            *refused = true;
            return Focus::Stay;
        }
        self.overlay = Overlay::Closed;
        self.edited(|_| true)
    }

    pub fn dismiss(&mut self) -> Focus {
        self.overlay = Overlay::Closed;
        Focus::Search
    }

    fn ask_name(&mut self, purpose: Purpose, text: &str) -> Focus {
        self.overlay = Overlay::Naming {
            purpose,
            text: text.to_owned(),
            refused: false,
        };
        Focus::Name
    }

    fn create_group(&mut self, name: &str, with: Option<String>) -> bool {
        let Some(group) = self.groups.create(name) else {
            return false;
        };
        if let Some(app) = with {
            self.groups.add(group, &app);
        }
        self.show_shelf(Shelf::Group(group));
        true
    }

    fn remove_group(&mut self, group: usize) -> bool {
        let removed = self.groups.remove(group).is_some();
        if removed {
            self.shelf = after_removal(self.shelf, group);
        }
        removed
    }

    fn edited(&mut self, change: impl FnOnce(&mut Self) -> bool) -> Focus {
        if change(self) {
            self.persist();
            self.settle();
        }
        Focus::Search
    }
}
