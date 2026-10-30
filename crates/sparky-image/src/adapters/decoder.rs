use std::fs;
use std::path::Path;

use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageError, ImageReader, Limits};

use crate::domain::picture::Picture;
use crate::ports::{DecodeError, Decoder, MAX_SIDE};

const MAX_ALLOC: u64 = 1024 * 1024 * 1024;

pub struct ImageFile;

impl Decoder for ImageFile {
    fn decode(&self, path: &Path) -> Result<Picture, DecodeError> {
        let metadata = fs::metadata(path).map_err(unreadable)?;
        if !metadata.is_file() {
            return Err(DecodeError::NotAFile);
        }
        let mut reader = ImageReader::open(path)
            .map_err(unreadable)?
            .with_guessed_format()
            .map_err(unreadable)?;
        reader.limits(limits());
        let mut decoder = reader.into_decoder().map_err(classify)?;
        let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
        let mut image = DynamicImage::from_decoder(decoder).map_err(classify)?;
        image.apply_orientation(orientation);
        let rgba = image.into_rgba8();
        Ok(Picture {
            width: rgba.width(),
            height: rgba.height(),
            rgba: rgba.into_raw(),
        })
    }
}

fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_ALLOC);
    limits
}

fn unreadable(error: std::io::Error) -> DecodeError {
    DecodeError::Unreadable(error.to_string())
}

fn classify(error: ImageError) -> DecodeError {
    match error {
        ImageError::Unsupported(_) => DecodeError::Unsupported,
        ImageError::Limits(_) => DecodeError::TooLarge,
        ImageError::IoError(error) => unreadable(error),
        ImageError::Decoding(_) | ImageError::Encoding(_) | ImageError::Parameter(_) => {
            DecodeError::Corrupted
        }
    }
}
