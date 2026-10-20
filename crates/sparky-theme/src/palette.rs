use std::path::{Path, PathBuf};

use crate::colors_file;
use crate::rgb::Rgb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub mode: Mode,
    pub background: Rgb,
    pub lighter_background: Rgb,
    pub foreground: Rgb,
    pub bright_foreground: Rgb,
    pub dark_foreground: Rgb,
    pub accent: Rgb,
    pub selection: Rgb,
    pub muted: Rgb,
    pub urgent: Rgb,
    pub cyan: Rgb,
}

impl Palette {
    pub const TOKYO_NIGHT: Self = Self {
        mode: Mode::Dark,
        background: Rgb::new(0x1a, 0x1b, 0x26),
        lighter_background: Rgb::new(0x24, 0x28, 0x3b),
        foreground: Rgb::new(0xa9, 0xb1, 0xd6),
        bright_foreground: Rgb::new(0xc0, 0xca, 0xf5),
        dark_foreground: Rgb::new(0x56, 0x5f, 0x89),
        accent: Rgb::new(0x7a, 0xa2, 0xf7),
        selection: Rgb::new(0x29, 0x2e, 0x42),
        muted: Rgb::new(0x41, 0x48, 0x68),
        urgent: Rgb::new(0xf7, 0x76, 0x8e),
        cyan: Rgb::new(0x44, 0x9d, 0xab),
    };

    pub fn load(read: impl Fn(&Path) -> Option<String>, candidates: &[PathBuf]) -> Self {
        candidates
            .iter()
            .find_map(|path| read(path))
            .map(|contents| Self::from_colors(&contents))
            .unwrap_or(Self::TOKYO_NIGHT)
    }

    pub fn from_colors(contents: &str) -> Self {
        let fallback = Self::TOKYO_NIGHT;
        let color = |key: &str, default: Rgb| {
            colors_file::value(contents, key)
                .and_then(Rgb::parse)
                .unwrap_or(default)
        };
        Self {
            mode: mode_of(contents),
            background: color("background", fallback.background),
            lighter_background: color("lighter_background", fallback.lighter_background),
            foreground: color("foreground", fallback.foreground),
            bright_foreground: color("bright_foreground", fallback.bright_foreground),
            dark_foreground: color("dark_foreground", fallback.dark_foreground),
            accent: color("accent", fallback.accent),
            selection: color("selection", fallback.selection),
            muted: color("muted", fallback.muted),
            urgent: color("red", fallback.urgent),
            cyan: color("cyan", fallback.cyan),
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::TOKYO_NIGHT
    }
}

fn mode_of(contents: &str) -> Mode {
    match colors_file::value(contents, "mode") {
        Some("light") => Mode::Light,
        _ => Mode::Dark,
    }
}

#[cfg(test)]
mod tests {
    use super::{Mode, Palette};
    use crate::rgb::Rgb;
    use std::path::{Path, PathBuf};

    const TOKYO_NIGHT: &str = include_str!("../../../data/themes/tokyo-night/colors.toml");

    #[test]
    fn a_paleta_embutida_e_igual_ao_tema_distribuido() {
        assert_eq!(Palette::from_colors(TOKYO_NIGHT), Palette::TOKYO_NIGHT);
    }

    #[test]
    fn le_o_modo_claro() {
        assert_eq!(Palette::from_colors("mode = \"light\"").mode, Mode::Light);
    }

    #[test]
    fn modo_ausente_ou_desconhecido_e_escuro() {
        assert_eq!(Palette::from_colors("").mode, Mode::Dark);
        assert_eq!(Palette::from_colors("mode = \"sepia\"").mode, Mode::Dark);
    }

    #[test]
    fn a_cor_de_alerta_vem_do_vermelho() {
        let palette = Palette::from_colors("red = \"#ff0000\"");
        assert_eq!(palette.urgent, Rgb::new(0xff, 0x00, 0x00));
    }

    #[test]
    fn chave_ausente_mantem_a_cor_do_tokyo_night() {
        let palette = Palette::from_colors("accent = \"#2222ff\"");
        assert_eq!(palette.accent, Rgb::new(0x22, 0x22, 0xff));
        assert_eq!(palette.background, Palette::TOKYO_NIGHT.background);
    }

    #[test]
    fn hexadecimal_invalido_mantem_a_cor_do_tokyo_night() {
        let palette = Palette::from_colors("accent = \"azul\"");
        assert_eq!(palette.accent, Palette::TOKYO_NIGHT.accent);
    }

    #[test]
    fn carrega_o_primeiro_arquivo_que_existe() {
        let candidatos = [PathBuf::from("/usuario/colors.toml"), PathBuf::from("/sistema/colors.toml")];
        let ler = |path: &Path| match path.starts_with("/sistema") {
            true => Some("accent = \"#010203\"".to_owned()),
            false => None,
        };
        assert_eq!(Palette::load(ler, &candidatos).accent, Rgb::new(1, 2, 3));
    }

    #[test]
    fn o_arquivo_do_usuario_tem_prioridade() {
        let candidatos = [PathBuf::from("/usuario/colors.toml"), PathBuf::from("/sistema/colors.toml")];
        let ler = |path: &Path| match path.starts_with("/usuario") {
            true => Some("accent = \"#0a0b0c\"".to_owned()),
            false => Some("accent = \"#010203\"".to_owned()),
        };
        assert_eq!(Palette::load(ler, &candidatos).accent, Rgb::new(10, 11, 12));
    }

    #[test]
    fn sem_arquivo_nenhum_fica_o_tokyo_night() {
        let candidatos = [PathBuf::from("/nada/colors.toml")];
        assert_eq!(Palette::load(|_| None, &candidatos), Palette::TOKYO_NIGHT);
    }
}
