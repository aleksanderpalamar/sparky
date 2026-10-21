use crate::domain::app::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    ByName,
    AsListed,
}

pub fn rank<'a>(apps: &'a [App], query: &str, order: Order) -> Vec<&'a App> {
    let wanted = query.trim().to_lowercase();
    let mut found: Vec<(u8, String, &App)> = apps
        .iter()
        .filter_map(|app| score(app, &wanted).map(|score| (score, tiebreak(app, order), app)))
        .collect();
    found.sort_by(|left, right| (left.0, &left.1).cmp(&(right.0, &right.1)));
    found.into_iter().map(|(_, _, app)| app).collect()
}

fn tiebreak(app: &App, order: Order) -> String {
    match order {
        Order::ByName => app.name.to_lowercase(),
        Order::AsListed => String::new(),
    }
}

fn score(app: &App, wanted: &str) -> Option<u8> {
    if wanted.is_empty() {
        return Some(0);
    }
    let name = app.name.to_lowercase();
    let words = || name.split([' ', '-', '_', '.']);
    let extra = app.keywords.iter().chain(app.detail.iter()).chain(std::iter::once(&app.id));
    match () {
        _ if name == wanted => Some(0),
        _ if name.starts_with(wanted) => Some(1),
        _ if words().any(|word| word.starts_with(wanted)) => Some(2),
        _ if name.contains(wanted) => Some(3),
        _ if extra.map(|text| text.to_lowercase()).any(|text| text.contains(wanted)) => Some(4),
        _ if in_order(wanted, &name) => Some(5),
        _ => None,
    }
}

fn in_order(wanted: &str, name: &str) -> bool {
    let mut letters = name.chars();
    wanted.chars().all(|letter| letters.any(|candidate| candidate == letter))
}
