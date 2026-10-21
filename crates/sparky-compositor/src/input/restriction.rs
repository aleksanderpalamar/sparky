#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Restriction {
    Free,
    Locked,
    Confined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landing {
    Inside,
    Outside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Move,
    Stay,
}

pub fn step(restriction: Restriction, landing: Landing) -> Step {
    match (restriction, landing) {
        (Restriction::Free, _) | (Restriction::Confined, Landing::Inside) => Step::Move,
        (Restriction::Locked, _) | (Restriction::Confined, Landing::Outside) => Step::Stay,
    }
}

#[cfg(test)]
mod tests {
    use super::{Landing, Restriction, Step, step};

    #[test]
    fn ponteiro_livre_sempre_anda() {
        assert_eq!(step(Restriction::Free, Landing::Outside), Step::Move);
    }

    #[test]
    fn ponteiro_travado_nunca_anda() {
        assert_eq!(step(Restriction::Locked, Landing::Inside), Step::Stay);
    }

    #[test]
    fn ponteiro_preso_so_anda_dentro_da_regiao() {
        assert_eq!(step(Restriction::Confined, Landing::Inside), Step::Move);
        assert_eq!(step(Restriction::Confined, Landing::Outside), Step::Stay);
    }
}
