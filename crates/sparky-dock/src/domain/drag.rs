const THRESHOLD: f32 = 6.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Drag {
    pub id: String,
    pub origin: f32,
    pub moved: bool,
}

impl Drag {
    pub fn new(id: &str, origin: f32) -> Self {
        Self {
            id: id.to_owned(),
            origin,
            moved: false,
        }
    }

    pub fn follow(&mut self, x: f32) {
        if (x - self.origin).abs() > THRESHOLD {
            self.moved = true;
        }
    }
}

pub fn nearest(x: f32, centers: &[f32]) -> usize {
    centers
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| (*left - x).abs().total_cmp(&(*right - x).abs()))
        .map_or(0, |(index, _)| index)
}

pub fn arranged(order: &[String], id: &str, to: usize) -> Option<Vec<String>> {
    let from = order.iter().position(|item| item == id)?;
    let mut moved = order.to_vec();
    let item = moved.remove(from);
    moved.insert(to.min(moved.len()), item);
    Some(moved)
}

pub fn before(order: &[String], id: &str, to: usize) -> Option<Option<String>> {
    let from = order.iter().position(|item| item == id)?;
    if from == to.min(order.len() - 1) {
        return None;
    }
    let moved = arranged(order, id, to)?;
    let at = moved.iter().position(|item| item == id)?;
    Some(moved.get(at + 1).cloned())
}
