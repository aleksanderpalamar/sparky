use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const UNITS: f64 = 120.0;
const MIN: f64 = 60.0;
const MAX: f64 = 480.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutputScale(u32);

impl OutputScale {
    pub const ONE: Self = Self(120);

    pub fn from_factor(factor: f64) -> Option<Self> {
        let units = (factor * UNITS).round();
        (MIN..=MAX).contains(&units).then_some(Self(units as u32))
    }

    pub fn units(self) -> u32 {
        self.0
    }

    pub fn factor(self) -> f64 {
        f64::from(self.0) / UNITS
    }
}

impl Default for OutputScale {
    fn default() -> Self {
        Self::ONE
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputRule {
    pub name: String,
    #[serde(default)]
    pub scale: OutputScale,
}

pub fn scale_for(rules: &[OutputRule], name: &str) -> OutputScale {
    rules.iter().rev().find(|rule| rule.name == name).map_or(OutputScale::ONE, |rule| rule.scale)
}

impl Serialize for OutputScale {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.factor())
    }
}

impl<'de> Deserialize<'de> for OutputScale {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(Factor)
    }
}

struct Factor;

impl Factor {
    fn scale<E: de::Error>(factor: f64) -> Result<OutputScale, E> {
        OutputScale::from_factor(factor).ok_or_else(|| E::custom(format!("escala {factor} fora de 0.5 a 4")))
    }
}

impl Visitor<'_> for Factor {
    type Value = OutputScale;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "uma escala entre 0.5 e 4")
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<OutputScale, E> {
        Self::scale(value)
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<OutputScale, E> {
        Self::scale(value as f64)
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<OutputScale, E> {
        Self::scale(value as f64)
    }
}
