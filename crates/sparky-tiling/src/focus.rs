use crate::ids::WindowId;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FocusHistory {
    recent_first: Vec<WindowId>,
}

impl FocusHistory {
    pub fn current(&self) -> Option<WindowId> {
        self.recent_first.first().copied()
    }

    pub fn touch(&mut self, window: WindowId) {
        self.recent_first.retain(|known| *known != window);
        self.recent_first.insert(0, window);
    }

    pub fn forget(&mut self, window: WindowId) {
        self.recent_first.retain(|known| *known != window);
    }
}

#[cfg(test)]
mod tests {
    use super::FocusHistory;
    use crate::ids::WindowId;

    #[test]
    fn a_ultima_tocada_e_a_atual() {
        let mut history = FocusHistory::default();
        history.touch(WindowId(1));
        history.touch(WindowId(2));
        assert_eq!(history.current(), Some(WindowId(2)));
    }

    #[test]
    fn tocar_de_novo_traz_para_a_frente_sem_repetir() {
        let mut history = FocusHistory::default();
        history.touch(WindowId(1));
        history.touch(WindowId(2));
        history.touch(WindowId(1));
        assert_eq!(history.current(), Some(WindowId(1)));
        history.forget(WindowId(1));
        assert_eq!(history.current(), Some(WindowId(2)));
        history.forget(WindowId(2));
        assert_eq!(history.current(), None);
    }

    #[test]
    fn esquecer_a_atual_devolve_o_foco_para_a_anterior() {
        let mut history = FocusHistory::default();
        history.touch(WindowId(1));
        history.touch(WindowId(2));
        history.forget(WindowId(2));
        assert_eq!(history.current(), Some(WindowId(1)));
    }

    #[test]
    fn historico_vazio_nao_tem_foco() {
        assert_eq!(FocusHistory::default().current(), None);
    }
}
