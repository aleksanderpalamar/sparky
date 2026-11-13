use crate::domain::undo::TrashEntry;

const LIFETIME: u8 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub names: Vec<String>,
    pub entries: Vec<TrashEntry>,
    ticks: u8,
}

impl Toast {
    pub fn trashed(names: Vec<String>, entries: Vec<TrashEntry>) -> Self {
        Self {
            names,
            entries,
            ticks: LIFETIME,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.ticks = self.ticks.saturating_sub(1);
        self.ticks > 0
    }
}

#[cfg(test)]
mod tests {
    use super::{LIFETIME, Toast};

    #[test]
    fn o_aviso_expira_depois_do_tempo_de_vida() {
        let mut toast = Toast::trashed(vec!["a.txt".to_owned()], Vec::new());
        let vivos = (0..LIFETIME).filter(|_| toast.tick()).count();
        assert_eq!(vivos, usize::from(LIFETIME) - 1);
        assert!(!toast.tick());
    }
}
