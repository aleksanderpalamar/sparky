use smithay::backend::renderer::Color32F;
use sparky_theme::{Palette, Rgb};

const INACTIVE: Color32F = Color32F::new(0x59 as f32 / 255.0, 0x59 as f32 / 255.0, 0x59 as f32 / 255.0, 0xaa as f32 / 255.0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderColors {
    pub active: Color32F,
    pub inactive: Color32F,
    pub background: Color32F,
}

impl BorderColors {
    pub fn from_palette(palette: Palette) -> Self {
        Self {
            active: opaque(palette.accent),
            inactive: INACTIVE,
            background: opaque(palette.background),
        }
    }

    pub fn for_focus(&self, focused: bool) -> Color32F {
        match focused {
            true => self.active,
            false => self.inactive,
        }
    }

    pub fn update(&mut self, palette: Palette) -> bool {
        let next = Self::from_palette(palette);
        if *self == next {
            return false;
        }
        *self = next;
        true
    }
}

fn opaque(rgb: Rgb) -> Color32F {
    Color32F::new(
        rgb.red as f32 / 255.0,
        rgb.green as f32 / 255.0,
        rgb.blue as f32 / 255.0,
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::BorderColors;
    use sparky_theme::Palette;

    #[test]
    fn a_borda_ativa_e_a_cor_de_destaque_do_tema() {
        let colors = BorderColors::from_palette(Palette::TOKYO_NIGHT);
        assert_eq!(colors.for_focus(true).r(), 0x7a as f32 / 255.0);
        assert_eq!(colors.for_focus(true).a(), 1.0);
    }

    #[test]
    fn a_borda_inativa_e_cinza_translucida() {
        let colors = BorderColors::from_palette(Palette::TOKYO_NIGHT);
        assert_eq!(colors.for_focus(false).a(), 0xaa as f32 / 255.0);
    }

    #[test]
    fn trocar_paleta_so_pede_redesenho_quando_as_cores_mudam() {
        let mut colors = BorderColors::from_palette(Palette::TOKYO_NIGHT);
        assert!(!colors.update(Palette::TOKYO_NIGHT));
        let palette = Palette::from_colors("accent = \"#010203\"\nbackground = \"#040506\"");
        assert!(colors.update(palette));
        assert_eq!(colors, BorderColors::from_palette(palette));
        assert!(!colors.update(palette));
    }
}
