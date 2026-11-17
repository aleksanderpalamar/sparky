use std::path::Path;

use image::imageops::FilterType;

use crate::domain::icon::Raster;

pub fn raster(path: &Path, pixels: u32) -> Option<Raster> {
    let decoded = image::open(path).ok()?;
    let fitted = match decoded.width().max(decoded.height()) > pixels {
        true => decoded.resize(pixels, pixels, FilterType::Lanczos3),
        false => decoded,
    };
    let rgba = fitted.to_rgba8();
    Some(Raster {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    })
}
