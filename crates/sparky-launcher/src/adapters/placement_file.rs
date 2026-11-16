use crate::domain::category::Category;
use crate::domain::placements::{Choice, Placement};

const ADDED: &str = "+";
const REMOVED: &str = "-";

pub fn read(contents: &str) -> Vec<Placement> {
    contents.lines().filter_map(parse).collect()
}

pub fn write(placements: &[Placement]) -> String {
    placements.iter().map(render).collect()
}

fn parse(line: &str) -> Option<Placement> {
    let mut fields = line.split('\t').map(str::trim);
    let choice = match fields.next()? {
        ADDED => Choice::Added,
        REMOVED => Choice::Removed,
        _ => return None,
    };
    let category = Category::from_key(fields.next()?)?;
    let app = fields.next().filter(|app| !app.is_empty())?;
    Some(Placement {
        category,
        app: app.to_owned(),
        choice,
    })
}

fn render(placement: &Placement) -> String {
    let choice = match placement.choice {
        Choice::Added => ADDED,
        Choice::Removed => REMOVED,
    };
    format!(
        "{choice}\t{}\t{}\n",
        placement.category.key(),
        placement.app
    )
}
