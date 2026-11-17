const LEGACY_ORDER: [&str; 8] = [
    "48x48", "64x64", "scalable", "32x32", "256x256", "24x24", "22x22", "16x16",
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Details {
    size: Option<u32>,
    scale: Option<u32>,
    scalable: bool,
}

impl Details {
    pub fn set(&mut self, key: &str, value: &str) {
        match key {
            "Size" => self.size = value.parse().ok(),
            "Scale" => self.scale = value.parse().ok(),
            "Type" => self.scalable = value == "Scalable",
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Scalable,
    Fixed { size: u32, scale: u32 },
    Unknown,
}

pub fn rank(directory: &str, details: Option<&Details>, pixels: Option<u32>) -> (u8, u32) {
    let Some(pixels) = pixels else {
        return legacy(directory);
    };
    match kind(directory, details) {
        Kind::Fixed { size, scale } => fixed(size, scale, pixels),
        Kind::Scalable => (2, 0),
        Kind::Unknown => (5, 0),
    }
}

fn fixed(size: u32, scale: u32, pixels: u32) -> (u8, u32) {
    let physical = size.saturating_mul(scale);
    match physical.cmp(&pixels) {
        std::cmp::Ordering::Equal if scale == 1 => (0, 0),
        std::cmp::Ordering::Equal => (1, 0),
        std::cmp::Ordering::Greater => (3, physical - pixels),
        std::cmp::Ordering::Less => (4, pixels - physical),
    }
}

fn kind(directory: &str, details: Option<&Details>) -> Kind {
    match details {
        Some(details) if details.scalable => Kind::Scalable,
        Some(Details {
            size: Some(size),
            scale,
            ..
        }) => Kind::Fixed {
            size: *size,
            scale: scale.unwrap_or(1),
        },
        _ => from_name(first_segment(directory)),
    }
}

fn from_name(segment: &str) -> Kind {
    if segment == "scalable" {
        return Kind::Scalable;
    }
    let (dimension, scale) = segment.split_once('@').unwrap_or((segment, "1x"));
    let parsed = dimension.split_once('x').and_then(|(width, height)| {
        let scale = scale.strip_suffix('x')?.parse::<u32>().ok()?;
        (width == height).then_some(())?;
        Some(Kind::Fixed {
            size: width.parse().ok()?,
            scale,
        })
    });
    parsed.unwrap_or(Kind::Unknown)
}

fn legacy(directory: &str) -> (u8, u32) {
    let size = first_segment(directory);
    let position = LEGACY_ORDER
        .iter()
        .position(|candidate| *candidate == size)
        .unwrap_or(LEGACY_ORDER.len());
    (0, position as u32)
}

fn first_segment(directory: &str) -> &str {
    directory.split('/').next().unwrap_or_default()
}
