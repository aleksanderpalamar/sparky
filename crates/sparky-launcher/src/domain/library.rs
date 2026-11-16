use crate::domain::app::App;
use crate::domain::category::CATEGORIES;
use crate::domain::groups::Groups;
use crate::domain::placements::Placements;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Library {
    pub groups: Groups,
    pub placements: Placements,
}

impl Library {
    pub fn categorized(&self, app: &App) -> bool {
        CATEGORIES
            .into_iter()
            .any(|category| self.placements.holds(category, app))
    }

    pub fn shelved(&self, app: &App) -> bool {
        self.groups.grouped(&app.id) || self.categorized(app)
    }
}
