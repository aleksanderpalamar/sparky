use crate::domain::app::App;
use crate::domain::category::{CATEGORIES, Category};
use crate::domain::library::Library;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shelf {
    #[default]
    Home,
    Category(Category),
    Group(usize),
}

impl Shelf {
    pub fn holds(self, app: &App, library: &Library) -> bool {
        match self {
            Shelf::Home => !library.shelved(app),
            Shelf::Category(category) => library.placements.holds(category, app),
            Shelf::Group(index) => library.groups.holds(index, &app.id),
        }
    }
}

pub fn shelves(apps: &[App], library: &Library) -> Vec<Shelf> {
    let categories = CATEGORIES
        .into_iter()
        .filter(|category| {
            apps.iter()
                .any(|app| library.placements.holds(*category, app))
        })
        .map(Shelf::Category);
    let custom = (0..library.groups.list().len()).map(Shelf::Group);
    std::iter::once(Shelf::Home)
        .chain(categories)
        .chain(custom)
        .collect()
}

pub fn neighbour(shelves: &[Shelf], current: Shelf, delta: isize) -> Shelf {
    let Some(position) = shelves.iter().position(|shelf| *shelf == current) else {
        return Shelf::Home;
    };
    let count = shelves.len() as isize;
    let next = (position as isize + delta).rem_euclid(count.max(1)) as usize;
    shelves.get(next).copied().unwrap_or_default()
}

pub fn after_removal(current: Shelf, removed: usize) -> Shelf {
    match current {
        Shelf::Group(index) if index == removed => Shelf::Home,
        Shelf::Group(index) if index > removed => Shelf::Group(index - 1),
        other => other,
    }
}
