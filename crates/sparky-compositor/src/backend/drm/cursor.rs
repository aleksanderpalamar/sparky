use std::env;
use std::fs;

use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::memory::MemoryRenderBuffer;
use smithay::utils::{Logical, Point, Transform};
use xcursor::CursorTheme;
use xcursor::parser::{Image, parse_xcursor};

use crate::backend::drm::arrow;

const DEFAULT_SIZE: u32 = 24;
const ICONS: [&str; 2] = ["left_ptr", "default"];

#[derive(Debug, Clone)]
pub struct CursorImage {
    pub buffer: MemoryRenderBuffer,
    pub hotspot: Point<i32, Logical>,
}

impl CursorImage {
    pub fn load() -> Self {
        let theme = env::var("XCURSOR_THEME").unwrap_or_else(|_| "default".to_owned());
        let size = env::var("XCURSOR_SIZE")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_SIZE);
        from_theme(&theme, size).unwrap_or_else(fallback)
    }
}

pub fn nearest(sizes: &[u32], wanted: u32) -> Option<usize> {
    sizes
        .iter()
        .enumerate()
        .min_by_key(|(_, size)| size.abs_diff(wanted))
        .map(|(index, _)| index)
}

fn from_theme(theme: &str, size: u32) -> Option<CursorImage> {
    let theme = CursorTheme::load(theme);
    let path = ICONS.iter().find_map(|icon| theme.load_icon(icon))?;
    let images = parse_xcursor(&fs::read(path).ok()?)?;
    let sizes: Vec<u32> = images.iter().map(|image| image.size).collect();
    let image = images.get(nearest(&sizes, size)?)?;
    Some(from_image(image))
}

fn from_image(image: &Image) -> CursorImage {
    CursorImage {
        buffer: MemoryRenderBuffer::from_slice(
            &image.pixels_rgba,
            Fourcc::Abgr8888,
            (image.width as i32, image.height as i32),
            1,
            Transform::Normal,
            None,
        ),
        hotspot: (image.xhot as i32, image.yhot as i32).into(),
    }
}

fn fallback() -> CursorImage {
    CursorImage {
        buffer: MemoryRenderBuffer::from_slice(
            &arrow::pixels(),
            Fourcc::Abgr8888,
            (arrow::WIDTH as i32, arrow::HEIGHT as i32),
            1,
            Transform::Normal,
            None,
        ),
        hotspot: (0, 0).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::nearest;

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
