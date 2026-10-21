pub mod arrow;

use std::env;
use std::fs;

use smithay::input::pointer::CursorIcon;
use xcursor::CursorTheme;
use xcursor::parser::{Image, parse_xcursor};

const DEFAULT_SIZE: u32 = 24;
const ICONS: [&str; 2] = ["left_ptr", "default"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorPixels {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub hotspot: (u32, u32),
}

impl CursorPixels {
    pub fn load() -> Self {
        Self::named(&ICONS).unwrap_or_else(arrow_cursor)
    }

    pub fn named(names: &[&str]) -> Option<Self> {
        let theme = env::var("XCURSOR_THEME").unwrap_or_else(|_| "default".to_owned());
        let size = env::var("XCURSOR_SIZE")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_SIZE);
        from_theme(&theme, size, names)
    }
}

pub fn names_for(icon: CursorIcon) -> Vec<&'static str> {
    std::iter::once(icon.name()).chain(icon.alt_names().iter().copied()).collect()
}

pub fn nearest(sizes: &[u32], wanted: u32) -> Option<usize> {
    sizes
        .iter()
        .enumerate()
        .min_by_key(|(_, size)| size.abs_diff(wanted))
        .map(|(index, _)| index)
}

fn from_theme(theme: &str, size: u32, names: &[&str]) -> Option<CursorPixels> {
    let theme = CursorTheme::load(theme);
    let path = names.iter().find_map(|icon| theme.load_icon(icon))?;
    let images = parse_xcursor(&fs::read(path).ok()?)?;
    let sizes: Vec<u32> = images.iter().map(|image| image.size).collect();
    let image = images.get(nearest(&sizes, size)?)?;
    Some(from_image(image))
}

fn from_image(image: &Image) -> CursorPixels {
    CursorPixels {
        rgba: image.pixels_rgba.clone(),
        width: image.width,
        height: image.height,
        hotspot: (image.xhot, image.yhot),
    }
}

fn arrow_cursor() -> CursorPixels {
    CursorPixels {
        rgba: arrow::pixels(),
        width: arrow::WIDTH as u32,
        height: arrow::HEIGHT as u32,
        hotspot: (0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::{names_for, nearest};
    use smithay::input::pointer::CursorIcon;

    #[test]
    fn o_formato_pedido_vem_primeiro_e_depois_os_nomes_antigos_do_tema() {
        let texto = names_for(CursorIcon::Text);
        assert_eq!(texto.first(), Some(&"text"));
        assert!(texto.contains(&"xterm"));
        assert!(names_for(CursorIcon::Default).contains(&"left_ptr"));
    }

    #[test]
    fn escolhe_o_tamanho_mais_proximo() {
        assert_eq!(nearest(&[16, 24, 32, 48], 24), Some(1));
        assert_eq!(nearest(&[16, 32, 48], 36), Some(1));
    }

    #[test]
    fn sem_tamanhos_nao_ha_imagem() {
        assert_eq!(nearest(&[], 24), None);
    }
}
