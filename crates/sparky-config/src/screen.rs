use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const TOLERANCE: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutputMode {
    pub width: u32,
    pub height: u32,
    pub refresh: u32,
}

impl OutputMode {
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim().trim_end_matches("Hz");
        let (size, refresh) = match text.split_once('@') {
            Some((size, refresh)) => (size, millihertz(refresh)?),
            None => (text, 0),
        };
        let (width, height) = size.split_once('x')?;
        let width = width.parse().ok().filter(|width| *width > 0)?;
        let height = height.parse().ok().filter(|height| *height > 0)?;
        Some(Self {
            width,
            height,
            refresh,
        })
    }

    pub fn label(&self) -> String {
        let centihertz = (self.refresh + 5) / 10;
        format!(
            "{}x{}@{}.{:02}Hz",
            self.width,
            self.height,
            centihertz / 100,
            centihertz % 100
        )
    }

    pub fn matches(&self, other: &OutputMode) -> bool {
        self.width == other.width
            && self.height == other.height
            && (self.refresh == 0 || self.refresh.abs_diff(other.refresh) <= TOLERANCE)
    }
}

impl fmt::Display for OutputMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}x{}", self.width, self.height)?;
        match self.refresh {
            0 => Ok(()),
            refresh => write!(formatter, "@{}.{:03}", refresh / 1000, refresh % 1000),
        }
    }
}

fn millihertz(text: &str) -> Option<u32> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) || fraction.len() > 3 {
        return None;
    }
    let fraction = format!("{fraction:0<3}");
    let whole: u32 = whole.parse().ok()?;
    let fraction: u32 = fraction.parse().ok()?;
    whole
        .checked_mul(1000)?
        .checked_add(fraction)
        .filter(|refresh| *refresh > 0)
}

impl Serialize for OutputMode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for OutputMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        OutputMode::parse(&text)
            .ok_or_else(|| de::Error::custom(format!("modo de vídeo inválido: {text}")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum Rotation {
    #[default]
    Normal,
    Right,
    Inverted,
    Left,
}

impl Rotation {
    pub const ALL: [Rotation; 4] = [
        Rotation::Normal,
        Rotation::Right,
        Rotation::Inverted,
        Rotation::Left,
    ];

    pub const fn degrees(self) -> u16 {
        match self {
            Rotation::Normal => 0,
            Rotation::Right => 90,
            Rotation::Inverted => 180,
            Rotation::Left => 270,
        }
    }

    pub fn from_degrees(degrees: u16) -> Option<Self> {
        Rotation::ALL
            .into_iter()
            .find(|rotation| rotation.degrees() == degrees)
    }

    pub const fn swaps_sides(self) -> bool {
        matches!(self, Rotation::Right | Rotation::Left)
    }

    pub fn is_normal(&self) -> bool {
        *self == Rotation::Normal
    }
}

impl Serialize for Rotation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.degrees())
    }
}

impl<'de> Deserialize<'de> for Rotation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(Degrees)
    }
}

struct Degrees;

impl Visitor<'_> for Degrees {
    type Value = Rotation;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "um giro de 0, 90, 180 ou 270 graus")
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Rotation, E> {
        u16::try_from(value)
            .ok()
            .and_then(Rotation::from_degrees)
            .ok_or_else(|| E::custom(format!("giro {value} fora de 0, 90, 180 ou 270")))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Rotation, E> {
        self.visit_i64(i64::try_from(value).unwrap_or(i64::MAX))
    }
}
