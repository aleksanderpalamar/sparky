use sparky_tiling::WindowId;

pub const NO_WINDOW: u32 = 0;

pub trait ActiveWindow {
    fn announce(&self, window: u32);
}

pub fn correction(current: Option<u32>, wanted: u32) -> Option<u32> {
    match current == Some(wanted) {
        true => None,
        false => Some(wanted),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Minimize {
    Refuse,
    Ignore,
}

pub fn minimize_answer(requested: WindowId, focused: Option<WindowId>) -> Minimize {
    match focused == Some(requested) {
        true => Minimize::Refuse,
        false => Minimize::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use sparky_tiling::WindowId;

    use super::{Minimize, NO_WINDOW, correction, minimize_answer};

    #[test]
    fn a_raiz_gravada_pelo_smithay_e_trocada_pela_janela_focada() {
        assert_eq!(correction(Some(0x408), 0x4a00001), Some(0x4a00001));
    }

    #[test]
    fn a_janela_certa_ja_anunciada_nao_e_regravada() {
        assert_eq!(correction(Some(0x4a00001), 0x4a00001), None);
        assert_eq!(correction(Some(NO_WINDOW), NO_WINDOW), None);
    }

    #[test]
    fn sem_valor_gravado_anuncia_a_janela_desejada() {
        assert_eq!(correction(None, NO_WINDOW), Some(NO_WINDOW));
        assert_eq!(correction(None, 0x4a00001), Some(0x4a00001));
    }

    #[test]
    fn foco_numa_janela_wayland_limpa_a_janela_ativa_do_x11() {
        assert_eq!(correction(Some(0x4a00001), NO_WINDOW), Some(NO_WINDOW));
    }

    #[test]
    fn a_janela_focada_que_pede_para_minimizar_e_recusada() {
        let jogo = WindowId(7);
        assert_eq!(minimize_answer(jogo, Some(jogo)), Minimize::Refuse);
    }

    #[test]
    fn a_janela_sem_foco_que_pede_para_minimizar_fica_como_esta() {
        assert_eq!(
            minimize_answer(WindowId(7), Some(WindowId(3))),
            Minimize::Ignore
        );
        assert_eq!(minimize_answer(WindowId(7), None), Minimize::Ignore);
    }
}
