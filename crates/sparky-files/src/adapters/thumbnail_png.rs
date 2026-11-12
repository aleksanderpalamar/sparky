use std::fs::{self, DirBuilder};
use std::io::{self, BufWriter, Cursor};
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;

use image::{ImageFormat, ImageReader, Limits};

use crate::domain::thumbnail::Thumbnail;

pub const EDGE: u32 = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    pub uri: String,
    pub mtime: u64,
    pub size: u64,
}

pub fn read(file: &Path, expected: &Stamp) -> Option<Thumbnail> {
    let bytes = fs::read(file).ok()?;
    let decoder = png::Decoder::new(Cursor::new(bytes.as_slice()));
    let reader = decoder.read_info().ok()?;
    let texts = &reader.info().uncompressed_latin1_text;
    let text = |key: &str| {
        texts
            .iter()
            .find(|chunk| chunk.keyword == key)
            .map(|chunk| chunk.text.as_str())
    };
    if text("Thumb::URI") != Some(expected.uri.as_str()) {
        return None;
    }
    if text("Thumb::MTime").and_then(|mtime| mtime.parse().ok()) != Some(expected.mtime) {
        return None;
    }
    decode(&bytes)
}

pub fn write(dir: &Path, name: &str, stamp: &Stamp, thumbnail: &Thumbnail) -> io::Result<()> {
    DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
    let staged = tempfile::Builder::new()
        .prefix(".sparky-files-")
        .suffix(".png")
        .tempfile_in(dir)?;
    encode(staged.as_file(), stamp, thumbnail).map_err(io::Error::other)?;
    staged
        .persist(dir.join(name))
        .map(|_| ())
        .map_err(|error| error.error)
}

fn decode(bytes: &[u8]) -> Option<Thumbnail> {
    let mut reader = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);
    let mut limits = Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().ok()?;
    let image = match image.width().max(image.height()) > EDGE {
        true => image.thumbnail(EDGE, EDGE),
        false => image,
    }
    .to_rgba8();
    Some(Thumbnail {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    })
}

fn encode(file: &fs::File, stamp: &Stamp, thumbnail: &Thumbnail) -> Result<(), png::EncodingError> {
    let mut encoder = png::Encoder::new(BufWriter::new(file), thumbnail.width, thumbnail.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.add_text_chunk("Thumb::URI".to_owned(), stamp.uri.clone())?;
    encoder.add_text_chunk("Thumb::MTime".to_owned(), stamp.mtime.to_string())?;
    encoder.add_text_chunk("Thumb::Size".to_owned(), stamp.size.to_string())?;
    encoder.add_text_chunk("Software".to_owned(), "sparky-files".to_owned())?;
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&thumbnail.rgba)?;
    writer.finish()
}
