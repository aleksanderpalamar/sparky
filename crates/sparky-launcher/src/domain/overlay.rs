use crate::domain::menu::Target;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Purpose {
    Create { with: Option<String> },
    Rename(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Menu {
    pub target: Target,
    pub point: (f32, f32),
    pub cursor: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Overlay {
    #[default]
    Closed,
    Menu(Menu),
    Naming {
        purpose: Purpose,
        text: String,
        refused: bool,
    },
}
