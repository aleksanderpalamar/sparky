use iced_core::Color;

use crate::palette::{Mode, Palette};
use crate::rgb::Rgb;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Skin {
    pub mode: Mode,
    pub background: Color,
    pub lighter_background: Color,
    pub foreground: Color,
    pub bright_foreground: Color,
    pub dark_foreground: Color,
    pub accent: Color,
    pub selection: Color,
    pub muted: Color,
    pub urgent: Color,
    pub cyan: Color,
}

impl Skin {
    pub fn new(palette: Palette) -> Self {
        Self {
            mode: palette.mode,
            background: convert(palette.background),
            lighter_background: convert(palette.lighter_background),
            foreground: convert(palette.foreground),
            bright_foreground: convert(palette.bright_foreground),
            dark_foreground: convert(palette.dark_foreground),
            accent: convert(palette.accent),
            selection: convert(palette.selection),
            muted: convert(palette.muted),
            urgent: convert(palette.urgent),
            cyan: convert(palette.cyan),
        }
    }

    pub fn fill(&self, alpha: f32) -> Color {
        with_alpha(self.foreground, alpha)
    }

    pub fn accent_fill(&self, alpha: f32) -> Color {
        with_alpha(self.accent, alpha)
    }
}

impl Default for Skin {
    fn default() -> Self {
        Self::new(Palette::TOKYO_NIGHT)
    }
}

fn convert(rgb: Rgb) -> Color {
    Color::from_rgb8(rgb.red, rgb.green, rgb.blue)
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color {
        a: alpha.clamp(0.0, 1.0),
        ..color
    }
}

#[cfg(test)]
mod tests {
    use super::Skin;
    use crate::palette::Palette;
    use iced_core::Color;

    #[test]
    fn converte_as_cores_da_paleta() {
        let skin = Skin::new(Palette::TOKYO_NIGHT);
        assert_eq!(skin.accent, Color::from_rgb8(0x7a, 0xa2, 0xf7));
        assert_eq!(skin.background, Color::from_rgb8(0x1a, 0x1b, 0x26));
    }

    #[test]
    fn o_preenchimento_usa_a_cor_de_frente_com_transparencia() {
        let skin = Skin::default();
        let fill = skin.fill(0.08);
        assert_eq!(fill.a, 0.08);
        assert_eq!(fill.r, skin.foreground.r);
    }

    #[test]
    fn o_preenchimento_de_destaque_usa_a_cor_de_destaque() {
        let skin = Skin::default();
        assert_eq!(skin.accent_fill(0.5).b, skin.accent.b);
    }

    #[test]
    fn a_transparencia_fica_entre_zero_e_um() {
        let skin = Skin::default();
        assert_eq!(skin.fill(-1.0).a, 0.0);
        assert_eq!(skin.fill(3.0).a, 1.0);
    }
}
