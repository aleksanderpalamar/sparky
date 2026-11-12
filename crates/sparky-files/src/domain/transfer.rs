use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Copy,
    Move,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conflict {
    KeepBoth,
    Replace,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub source: PathBuf,
    pub target: PathBuf,
    pub action: Action,
    pub replace: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Progress {
    pub items_done: usize,
    pub items_total: usize,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

impl Progress {
    pub fn percent(&self) -> u8 {
        if self.bytes_total == 0 {
            return match self.items_total {
                0 => 100,
                total => (self.items_done * 100 / total) as u8,
            };
        }
        (self.bytes_done.min(self.bytes_total) * 100 / self.bytes_total) as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ending {
    Finished,
    Cancelled,
    Failed { name: String, error: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub completed: Vec<Step>,
    pub skipped: usize,
    pub ending: Ending,
}

#[cfg(test)]
mod tests {
    use super::Progress;

    #[test]
    fn porcentagem_segue_os_bytes_e_sem_bytes_segue_os_itens() {
        let bytes = Progress {
            items_done: 0,
            items_total: 2,
            bytes_done: 50,
            bytes_total: 200,
        };
        assert_eq!(bytes.percent(), 25);
        let itens = Progress {
            items_done: 1,
            items_total: 4,
            ..Progress::default()
        };
        assert_eq!(itens.percent(), 25);
        assert_eq!(Progress::default().percent(), 100);
    }
}
