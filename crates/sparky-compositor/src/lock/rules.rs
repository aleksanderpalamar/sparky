#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Open,
    Locked,
}

pub fn accepts(phase: Phase, live_surfaces: usize) -> bool {
    match phase {
        Phase::Open => true,
        Phase::Locked => live_surfaces == 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{Phase, accepts};

    #[test]
    fn sessao_aberta_aceita_bloquear() {
        assert!(accepts(Phase::Open, 0));
    }

    #[test]
    fn com_um_locker_vivo_o_segundo_e_recusado() {
        assert!(!accepts(Phase::Locked, 1));
    }

    #[test]
    fn se_o_locker_morreu_um_novo_retoma_o_bloqueio() {
        assert!(accepts(Phase::Locked, 0));
    }
}
