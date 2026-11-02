use std::path::Path;

use iced::widget::image::Handle;
use image::imageops::{self, FilterType};

const BLURRED_HEIGHT: u32 = 540;
const SIGMA: f32 = 1.0;

#[derive(Debug, Clone)]
pub struct Backdrop {
    pub sharp: Handle,
    pub size: (u32, u32),
    pub blurred: Handle,
    pub reduction: f32,
}

pub fn load(path: &Path) -> Result<Backdrop, String> {
    let picture = decode(path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .into_rgba8();
    let (width, height) = picture.dimensions();
    let reduction = (BLURRED_HEIGHT as f32 / height.max(1) as f32).min(1.0);
    let reduce = |side: u32| ((side as f32 * reduction).round() as u32).max(1);
    let small = imageops::resize(
        &picture,
        reduce(width),
        reduce(height),
        FilterType::Triangle,
    );
    let blurred = imageops::fast_blur(&small, SIGMA);
    Ok(Backdrop {
        blurred: Handle::from_rgba(blurred.width(), blurred.height(), blurred.into_raw()),
        sharp: Handle::from_rgba(width, height, picture.into_raw()),
        size: (width, height),
        reduction,
    })
}

fn decode(path: &Path) -> image::ImageResult<image::DynamicImage> {
    image::ImageReader::open(path)?
        .with_guessed_format()?
        .decode()
}
