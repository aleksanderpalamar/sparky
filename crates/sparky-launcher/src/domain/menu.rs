use crate::domain::app::App;
use crate::domain::category::{CATEGORIES, Category};
use crate::domain::library::Library;
use crate::domain::shelf::Shelf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    App(App),
    Group(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    Category(Category),
    Group(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    AddTo { spot: Spot, app: App },
    TakeFrom { spot: Spot, app: App },
    NewGroupWith(String),
    Rename(usize),
    Remove(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub action: Action,
    pub label: &'static str,
    pub spot: Option<Spot>,
}

pub fn items(target: &Target, library: &Library, shelf: Shelf) -> Vec<Item> {
    match target {
        Target::App(app) => app_items(app, library, shelf),
        Target::Group(index) => group_items(*index, library),
    }
}

fn app_items(app: &App, library: &Library, shelf: Shelf) -> Vec<Item> {
    let holds = |spot: Spot| match spot {
        Spot::Category(category) => library.placements.holds(category, app),
        Spot::Group(index) => library.groups.holds(index, &app.id),
    };
    let current = match shelf {
        Shelf::Category(category) => Some(Spot::Category(category)),
        Shelf::Group(index) => Some(Spot::Group(index)),
        Shelf::Home => None,
    };
    let take = current.filter(|spot| holds(*spot)).map(|spot| Item {
        action: Action::TakeFrom {
            spot,
            app: app.clone(),
        },
        label: "menu-take-from",
        spot: Some(spot),
    });
    let spots = CATEGORIES
        .into_iter()
        .map(Spot::Category)
        .chain((0..library.groups.list().len()).map(Spot::Group));
    let add = spots.filter(|spot| !holds(*spot)).map(|spot| Item {
        action: Action::AddTo {
            spot,
            app: app.clone(),
        },
        label: "menu-add-to",
        spot: Some(spot),
    });
    let create = Item {
        action: Action::NewGroupWith(app.id.clone()),
        label: "menu-new-group-with",
        spot: None,
    };
    take.into_iter()
        .chain(add)
        .chain(std::iter::once(create))
        .collect()
}

fn group_items(index: usize, library: &Library) -> Vec<Item> {
    if library.groups.get(index).is_none() {
        return Vec::new();
    }
    vec![
        Item {
            action: Action::Rename(index),
            label: "menu-rename-group",
            spot: None,
        },
        Item {
            action: Action::Remove(index),
            label: "menu-remove-group",
            spot: None,
        },
    ]
}

pub fn placed(point: (f32, f32), area: (f32, f32), menu: (f32, f32)) -> (f32, f32) {
    let fit = |start: f32, room: f32, size: f32| start.min(room - size).max(0.0);
    (fit(point.0, area.0, menu.0), fit(point.1, area.1, menu.1))
}
