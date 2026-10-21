use std::path::PathBuf;

use smithay::utils::{Logical, Size};
use sparky_theme::wallpapers;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Choice {
    #[default]
    Off,
    File(PathBuf),
}

pub fn choice(configured: Option<&str>, home: &str, fallback: impl FnOnce() -> Option<PathBuf>) -> Choice {
    match configured.map(str::trim) {
        Some("") => Choice::Off,
        Some(path) => Choice::File(wallpapers::expand(path, home)),
        None => fallback().map_or(Choice::Off, Choice::File),
    }
}

pub fn physical(size: Size<i32, Logical>, scale: f64) -> (u32, u32) {
    let side = |length: i32| ((f64::from(length) * scale).round() as u32).max(1);
    (side(size.w), side(size.h))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use smithay::utils::Size;

    use super::{Choice, choice, physical};

    fn padrao() -> Option<PathBuf> {
        Some(PathBuf::from("/usr/share/sparky/wallpapers/tokyo-night/1-lago.webp"))
    }

    #[test]
    fn sem_escolha_vale_o_papel_de_parede_do_tema_padrao() {
        assert_eq!(choice(None, "/home/ana", padrao), Choice::File(PathBuf::from("/usr/share/sparky/wallpapers/tokyo-night/1-lago.webp")));
        assert_eq!(choice(None, "/home/ana", || None), Choice::Off);
    }

    #[test]
    fn texto_vazio_desliga_e_caminho_com_til_vai_para_a_pasta_pessoal() {
        assert_eq!(choice(Some(" "), "/home/ana", padrao), Choice::Off);
        assert_eq!(choice(Some("~/fundo.jpg"), "/home/ana", padrao), Choice::File(PathBuf::from("/home/ana/fundo.jpg")));
    }

    #[test]
    fn o_tamanho_em_pixels_segue_a_escala_do_monitor() {
        assert_eq!(physical(Size::from((1280, 720)), 1.5), (1920, 1080));
        assert_eq!(physical(Size::from((0, 0)), 1.0), (1, 1));
    }
}
