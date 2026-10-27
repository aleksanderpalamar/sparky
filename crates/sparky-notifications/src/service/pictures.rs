use crate::domain::Picture;
use std::{collections::HashMap, path::PathBuf};
use zbus::zvariant::OwnedValue;

type Raw = (i32, i32, i32, bool, i32, i32, Vec<u8>);

pub struct Prepared {
    data: Option<Raw>,
    path: Option<PathBuf>,
}

pub fn prepare(hints: &HashMap<String, OwnedValue>, icon: &str) -> Prepared {
    let data = ["image-data", "image_data", "icon_data"]
        .iter()
        .find_map(|key| hints.get(*key)?.try_clone().ok()?.try_into().ok());
    let path = ["image-path", "image_path"]
        .iter()
        .find_map(|key| <&str>::try_from(hints.get(*key)?).ok())
        .unwrap_or(icon);
    Prepared {
        data,
        path: file_path(path),
    }
}

fn file_path(value: &str) -> Option<PathBuf> {
    if value.starts_with('/') {
        return Some(PathBuf::from(value));
    }
    url::Url::parse(value).ok()?.to_file_path().ok()
}

impl Prepared {
    pub fn resolve(self) -> Option<Picture> {
        match self.data {
            Some(data) => from_raw(data),
            None => from_file(self.path?),
        }
    }
}

pub fn file(path: &str) -> Option<Picture> {
    from_file(file_path(path)?)
}

fn from_file(path: PathBuf) -> Option<Picture> {
    let metadata = std::fs::metadata(&path).ok()?;
    if !metadata.is_file() || metadata.len() > 4 * 1024 * 1024 {
        return None;
    }
    let reader = image::ImageReader::open(&path)
        .ok()?
        .with_guessed_format()
        .ok()?;
    let (width, height) = reader.into_dimensions().ok()?;
    if width == 0 || height == 0 || u64::from(width) * u64::from(height) > 4 * 1024 * 1024 {
        return None;
    }
    let mut reader = image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?;
    let mut limits = image::Limits::default();
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().ok()?.thumbnail(64, 64).into_rgba8();
    Some(Picture {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    })
}

fn from_raw((width, height, stride, alpha, bits, channels, data): Raw) -> Option<Picture> {
    if width <= 0
        || height <= 0
        || stride <= 0
        || bits != 8
        || channels != if alpha { 4 } else { 3 }
    {
        return None;
    }
    let (width, height, stride, channels) = (
        width as usize,
        height as usize,
        stride as usize,
        channels as usize,
    );
    if width.checked_mul(height)? > 4 * 1024 * 1024 || stride < width.checked_mul(channels)? {
        return None;
    }
    let needed = (height - 1)
        .checked_mul(stride)?
        .checked_add(width.checked_mul(channels)?)?;
    if data.len() < needed {
        return None;
    }
    let scale = (64.0 / width as f64).min(64.0 / height as f64).min(1.0);
    let w = (width as f64 * scale).max(1.0) as usize;
    let h = (height as f64 * scale).max(1.0) as usize;
    let mut rgba = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        for x in 0..w {
            let index = (y * height / h) * stride + (x * width / w) * channels;
            rgba.extend_from_slice(&data[index..index + 3]);
            rgba.push(if alpha { data[index + 3] } else { 255 });
        }
    }
    Some(Picture {
        width: w as u32,
        height: h as u32,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::from_raw;

    #[test]
    fn imagem_respeita_stride_alpha_e_limite_de_miniatura() {
        let picture = from_raw((1, 2, 4, false, 8, 3, vec![255, 0, 0, 0, 0, 255, 0])).unwrap();
        assert_eq!(picture.rgba, [255, 0, 0, 255, 0, 255, 0, 255]);
        let picture = from_raw((128, 128, 512, true, 8, 4, vec![255; 128 * 128 * 4])).unwrap();
        assert_eq!((picture.width, picture.height), (64, 64));
        assert!(from_raw((100, 100, 300, false, 8, 3, vec![0; 10])).is_none());
        assert!(from_raw((-1, 1, 3, false, 8, 3, vec![0; 3])).is_none());
    }
}
