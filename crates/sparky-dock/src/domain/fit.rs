use super::layout::Layout;

pub const FLOOR: i32 = 24;
pub const EMERGENCY: i32 = 12;
const MOST: usize = 1024;

pub fn fitted(preferred: i32, pinned: usize, output: Option<u32>) -> Layout {
    let Some(limit) = output else {
        return Layout::new(preferred);
    };
    (EMERGENCY..=preferred.max(FLOOR))
        .rev()
        .map(Layout::new)
        .find(|layout| fits(layout, pinned, limit))
        .unwrap_or_else(|| Layout::new(EMERGENCY))
}

pub fn capacity(output: Option<u32>) -> usize {
    let Some(limit) = output else {
        return usize::MAX;
    };
    let smallest = Layout::new(FLOOR);
    (0..MOST)
        .take_while(|pinned| fits(&smallest, *pinned, limit))
        .count()
        .saturating_sub(1)
}

fn fits(layout: &Layout, pinned: usize, limit: u32) -> bool {
    layout.frame(pinned, false).size.0 <= limit
}
