pub const MIN_SCALE: u32 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransferFunction {
    Srgb,
    Gamma22,
    ExtLinear,
    St2084Pq,
}

impl TransferFunction {
    pub const ALL: [Self; 4] = [Self::Srgb, Self::Gamma22, Self::ExtLinear, Self::St2084Pq];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedPrimaries {
    Srgb,
    Bt2020,
    DisplayP3,
    DciP3,
}

impl NamedPrimaries {
    pub const ALL: [Self; 4] = [Self::Srgb, Self::Bt2020, Self::DisplayP3, Self::DciP3];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Xy {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Chromaticities {
    pub red: Xy,
    pub green: Xy,
    pub blue: Xy,
    pub white: Xy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Luminances {
    pub min: u32,
    pub max: u32,
    pub reference: u32,
}

impl Luminances {
    pub const fn range(self) -> Range {
        Range {
            min: self.min,
            max: self.max,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    pub min: u32,
    pub max: u32,
}

impl Range {
    pub fn from_nits(min: f64, max: f64) -> Self {
        Self {
            min: (min * f64::from(MIN_SCALE)).round() as u32,
            max: max.round() as u32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Target {
    pub primaries: Chromaticities,
    pub luminance: Range,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ContentLight {
    pub max_cll: Option<u32>,
    pub max_fall: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageDescription {
    pub transfer: TransferFunction,
    pub primaries: NamedPrimaries,
    pub luminances: Luminances,
    pub target: Target,
    pub light: ContentLight,
}

impl ImageDescription {
    pub const fn standard(transfer: TransferFunction, primaries: NamedPrimaries) -> Self {
        let luminances = transfer.default_luminances();
        Self {
            transfer,
            primaries,
            luminances,
            target: Target {
                primaries: primaries.chromaticities(),
                luminance: luminances.range(),
            },
            light: ContentLight {
                max_cll: None,
                max_fall: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Range;

    #[test]
    fn brilho_em_nits_vira_as_unidades_do_protocolo() {
        assert_eq!(
            Range::from_nits(0.005, 1_000.0),
            Range {
                min: 50,
                max: 1_000
            }
        );
        assert_eq!(
            Range::from_nits(0.0499, 672.7),
            Range { min: 499, max: 673 }
        );
    }
}
