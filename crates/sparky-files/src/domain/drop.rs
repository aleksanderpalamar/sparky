use crate::domain::transfer::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropTarget {
    Entry(usize),
    Place(usize),
    Tab(usize),
    Trash,
    Bookmarks,
    Bookmark(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Override {
    #[default]
    None,
    Copy,
    Move,
}

impl Override {
    pub fn from_keys(control: bool, shift: bool) -> Self {
        match (control, shift) {
            (true, _) => Override::Copy,
            (false, true) => Override::Move,
            (false, false) => Override::None,
        }
    }
}

pub fn action(held: Override, same_device: bool) -> Action {
    match (held, same_device) {
        (Override::Copy, _) => Action::Copy,
        (Override::Move, _) => Action::Move,
        (Override::None, true) => Action::Move,
        (Override::None, false) => Action::Copy,
    }
}

#[cfg(test)]
mod tests {
    use super::{Override, action};
    use crate::domain::transfer::Action;

    #[test]
    fn no_mesmo_disco_move_e_entre_discos_copia() {
        assert_eq!(action(Override::None, true), Action::Move);
        assert_eq!(action(Override::None, false), Action::Copy);
    }

    #[test]
    fn ctrl_forca_copia_e_shift_forca_mover() {
        assert_eq!(Override::from_keys(true, false), Override::Copy);
        assert_eq!(Override::from_keys(true, true), Override::Copy);
        assert_eq!(Override::from_keys(false, true), Override::Move);
        assert_eq!(Override::from_keys(false, false), Override::None);
        assert_eq!(action(Override::Copy, true), Action::Copy);
        assert_eq!(action(Override::Move, false), Action::Move);
    }
}
