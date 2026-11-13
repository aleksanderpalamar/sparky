use std::path::{Path, PathBuf};

use crate::domain::undo::TrashEntry;

const LIFETIME: u8 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToastKind {
    Trashed {
        names: Vec<String>,
        entries: Vec<TrashEntry>,
    },
    Restored(Vec<PathBuf>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub kind: ToastKind,
    ticks: u8,
}

impl Toast {
    pub fn trashed(names: Vec<String>, entries: Vec<TrashEntry>) -> Self {
        Self::new(ToastKind::Trashed { names, entries })
    }

    pub fn restored(paths: Vec<PathBuf>) -> Self {
        Self::new(ToastKind::Restored(paths))
    }

    fn new(kind: ToastKind) -> Self {
        Self {
            kind,
            ticks: LIFETIME,
        }
    }

    pub fn tick(&mut self) -> bool {
        self.ticks = self.ticks.saturating_sub(1);
        self.ticks > 0
    }

    pub fn folder(&self) -> Option<&Path> {
        let ToastKind::Restored(paths) = &self.kind else {
            return None;
        };
        let first = paths.first()?.parent()?;
        paths
            .iter()
            .all(|path| path.parent() == Some(first))
            .then_some(first)
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{LIFETIME, Toast};

    #[test]
    fn o_aviso_expira_depois_do_tempo_de_vida() {
        let mut toast = Toast::trashed(vec!["a.txt".to_owned()], Vec::new());
        let vivos = (0..LIFETIME).filter(|_| toast.tick()).count();
        assert_eq!(vivos, usize::from(LIFETIME) - 1);
        assert!(!toast.tick());
    }

    #[test]
    fn a_pasta_so_existe_quando_todos_voltaram_para_o_mesmo_lugar() {
        let juntos = Toast::restored(vec![PathBuf::from("/h/F/a"), PathBuf::from("/h/F/b")]);
        assert_eq!(juntos.folder(), Some(Path::new("/h/F")));
        let separados = Toast::restored(vec![PathBuf::from("/h/F/a"), PathBuf::from("/h/G/b")]);
        assert_eq!(separados.folder(), None);
        let lixeira = Toast::trashed(vec!["a".to_owned()], Vec::new());
        assert_eq!(lixeira.folder(), None);
    }
}
