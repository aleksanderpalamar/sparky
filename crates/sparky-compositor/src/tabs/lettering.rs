use std::collections::HashMap;

use ab_glyph::{Font, FontVec, PxScale, ScaleFont, point};
use fontdb::{Database, Family, Query, Weight};
use smithay::backend::allocator::Fourcc;
use smithay::backend::renderer::element::memory::MemoryRenderBuffer;
use smithay::utils::Transform;

use crate::tabs::fit::fit;

const SIZE: f32 = 12.0;
const PADDING: f32 = 6.0;
const CACHE_LIMIT: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Emphasis {
    Active,
    Inactive,
}

impl Emphasis {
    pub fn of(active: bool) -> Self {
        match active {
            true => Emphasis::Active,
            false => Emphasis::Inactive,
        }
    }

    fn color(self) -> [f32; 4] {
        match self {
            Emphasis::Active => [1.0, 1.0, 1.0, 1.0],
            Emphasis::Inactive => [1.0, 1.0, 1.0, 0x90 as f32 / 255.0],
        }
    }
}

struct Fonts {
    regular: FontVec,
    bold: Option<FontVec>,
}

enum Loading {
    Pending,
    Ready(Fonts),
    Missing,
}

pub struct Lettering {
    fonts: Loading,
    cache: HashMap<(String, (i32, i32), Emphasis), MemoryRenderBuffer>,
}

impl Default for Lettering {
    fn default() -> Self {
        Self { fonts: Loading::Pending, cache: HashMap::new() }
    }
}

impl Lettering {
    pub fn label(&mut self, text: &str, size: (i32, i32), emphasis: Emphasis) -> Option<&MemoryRenderBuffer> {
        let key = (text.to_owned(), size, emphasis);
        if !self.cache.contains_key(&key) {
            let pixels = self.render(text, size, emphasis)?;
            if self.cache.len() > CACHE_LIMIT {
                self.cache.clear();
            }
            let buffer = MemoryRenderBuffer::from_slice(&pixels, Fourcc::Abgr8888, size, 1, Transform::Normal, None);
            self.cache.insert(key.clone(), buffer);
        }
        self.cache.get(&key)
    }

    fn render(&mut self, text: &str, size: (i32, i32), emphasis: Emphasis) -> Option<Vec<u8>> {
        let fonts = self.fonts()?;
        let font = match (emphasis, &fonts.bold) {
            (Emphasis::Active, Some(bold)) => bold,
            _ => &fonts.regular,
        };
        let shown = fit(text, size.0 as f32 - 2.0 * PADDING, |candidate| measure(font, candidate));
        Some(draw(font, &shown, size, emphasis.color()))
    }

    fn fonts(&mut self) -> Option<&Fonts> {
        if matches!(self.fonts, Loading::Pending) {
            self.fonts = load().map_or(Loading::Missing, Loading::Ready);
        }
        match &self.fonts {
            Loading::Ready(fonts) => Some(fonts),
            Loading::Pending | Loading::Missing => None,
        }
    }
}

pub fn pixel(color: [f32; 4], coverage: f32) -> [u8; 4] {
    let alpha = color[3] * coverage.clamp(0.0, 1.0);
    let channel = |value: f32| (value * alpha * 255.0).round() as u8;
    [channel(color[0]), channel(color[1]), channel(color[2]), (alpha * 255.0).round() as u8]
}

const FAMILIES: [Family<'static>; 2] = [Family::Name(sparky_theme::FONT_FAMILY), Family::Monospace];

fn load() -> Option<Fonts> {
    let mut database = Database::new();
    database.load_system_fonts();
    let face = |weight: Weight| {
        let query = Query { families: &FAMILIES, weight, ..Query::default() };
        let id = database.query(&query)?;
        database
            .with_face_data(id, |data, index| FontVec::try_from_vec_and_index(data.to_vec(), index).ok())
            .flatten()
    };
    Some(Fonts { regular: face(Weight::NORMAL)?, bold: face(Weight::BOLD) })
}

fn measure(font: &FontVec, text: &str) -> f32 {
    let scaled = font.as_scaled(PxScale::from(SIZE));
    text.chars().map(|character| scaled.h_advance(scaled.glyph_id(character))).sum()
}

fn draw(font: &FontVec, text: &str, size: (i32, i32), color: [f32; 4]) -> Vec<u8> {
    let (width, height) = (size.0.max(1) as usize, size.1.max(1) as usize);
    let mut coverage = vec![0.0_f32; width * height];
    let scaled = font.as_scaled(PxScale::from(SIZE));
    let baseline = (height as f32 + scaled.ascent() + scaled.descent()) / 2.0;
    let mut caret = ((width as f32 - measure(font, text)) / 2.0).max(PADDING);
    for character in text.chars() {
        let id = scaled.glyph_id(character);
        let glyph = id.with_scale_and_position(SIZE, point(caret, baseline));
        caret += scaled.h_advance(id);
        let Some(outline) = font.outline_glyph(glyph) else {
            continue;
        };
        let bounds = outline.px_bounds();
        outline.draw(|x, y, amount| {
            let (px, py) = (bounds.min.x as i32 + x as i32, bounds.min.y as i32 + y as i32);
            if (0..width as i32).contains(&px) && (0..height as i32).contains(&py) {
                let index = py as usize * width + px as usize;
                coverage[index] = coverage[index].max(amount);
            }
        });
    }
    coverage.into_iter().flat_map(|amount| pixel(color, amount)).collect()
}

#[cfg(test)]
mod tests {
    use super::pixel;

    #[test]
    fn as_abas_preferem_a_fonte_do_sparky_e_caem_na_monoespacada() {
        assert_eq!(super::FAMILIES, [fontdb::Family::Name("JetBrainsMono Nerd Font"), fontdb::Family::Monospace]);
    }

    #[test]
    fn a_cor_sai_pre_multiplicada_pela_cobertura() {
        assert_eq!(pixel([1.0, 1.0, 1.0, 1.0], 1.0), [255, 255, 255, 255]);
        assert_eq!(pixel([1.0, 0.5, 0.0, 1.0], 0.5), [128, 64, 0, 128]);
        assert_eq!(pixel([1.0, 1.0, 1.0, 0.5], 0.0), [0, 0, 0, 0]);
    }

    #[test]
    fn cobertura_fora_da_faixa_e_limitada() {
        assert_eq!(pixel([1.0, 1.0, 1.0, 1.0], 3.0), [255, 255, 255, 255]);
    }
}
