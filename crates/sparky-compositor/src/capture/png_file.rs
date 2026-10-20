use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::capture::area::CaptureError;
use crate::capture::render::Image;

pub fn write(path: &Path, image: &Image) -> Result<(), CaptureError> {
    let file = File::create(path).map_err(failure)?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(failure)?;
    writer.write_image_data(&image.rgba).map_err(failure)?;
    writer.finish().map_err(failure)
}

fn failure(error: impl std::fmt::Display) -> CaptureError {
    CaptureError::Write(error.to_string())
}
