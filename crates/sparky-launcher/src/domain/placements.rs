use crate::domain::app::App;
use crate::domain::category::Category;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Added,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    pub category: Category,
    pub app: String,
    pub choice: Choice,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Placements {
    list: Vec<Placement>,
}

impl Placements {
    pub fn new(list: Vec<Placement>) -> Self {
        let mut placements = Self::default();
        for placement in list {
            placements.forget(placement.category, &placement.app);
            placements.list.push(placement);
        }
        placements
    }

    pub fn list(&self) -> &[Placement] {
        &self.list
    }

    pub fn holds(&self, category: Category, app: &App) -> bool {
        match self.choice(category, &app.id) {
            Some(Choice::Added) => true,
            Some(Choice::Removed) => false,
            None => category.holds(app),
        }
    }

    pub fn place(&mut self, category: Category, app: &App, choice: Choice) -> bool {
        let wanted = choice == Choice::Added;
        if self.holds(category, app) == wanted {
            return false;
        }
        self.forget(category, &app.id);
        if category.holds(app) != wanted {
            self.list.push(Placement {
                category,
                app: app.id.clone(),
                choice,
            });
        }
        true
    }

    fn choice(&self, category: Category, app: &str) -> Option<Choice> {
        self.list
            .iter()
            .find(|placement| placement.category == category && placement.app == app)
            .map(|placement| placement.choice)
    }

    fn forget(&mut self, category: Category, app: &str) {
        self.list
            .retain(|placement| placement.category != category || placement.app != app);
    }
}
