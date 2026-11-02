use super::description::{
    Chromaticities, ContentLight, ImageDescription, Luminances, MIN_SCALE, NamedPrimaries, Range,
    Target, TransferFunction,
};
use super::standards::PQ_SWING;

const UNKNOWN: u32 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamsError {
    IncompleteSet,
    AlreadySet,
    InvalidLuminance,
}

#[derive(Debug, Default)]
pub struct Params {
    transfer: Option<TransferFunction>,
    primaries: Option<NamedPrimaries>,
    luminances: Option<Luminances>,
    mastering_primaries: Option<Chromaticities>,
    mastering_luminance: Option<Range>,
    max_cll: Option<u32>,
    max_fall: Option<u32>,
}

impl Params {
    pub fn set_transfer(&mut self, transfer: TransferFunction) -> Result<(), ParamsError> {
        fill(&mut self.transfer, transfer)
    }

    pub fn set_primaries(&mut self, primaries: NamedPrimaries) -> Result<(), ParamsError> {
        fill(&mut self.primaries, primaries)
    }

    pub fn set_luminances(&mut self, luminances: Luminances) -> Result<(), ParamsError> {
        let ordered = brighter(luminances.min, luminances.max)
            && brighter(luminances.min, luminances.reference);
        if !ordered {
            return Err(ParamsError::InvalidLuminance);
        }
        fill(&mut self.luminances, luminances)
    }

    pub fn set_mastering_primaries(
        &mut self,
        primaries: Chromaticities,
    ) -> Result<(), ParamsError> {
        fill(&mut self.mastering_primaries, primaries)
    }

    pub fn set_mastering_luminance(&mut self, luminance: Range) -> Result<(), ParamsError> {
        if known_range(&luminance) && !brighter(luminance.min, luminance.max) {
            return Err(ParamsError::InvalidLuminance);
        }
        fill(&mut self.mastering_luminance, luminance)
    }

    pub fn set_max_cll(&mut self, value: u32) -> Result<(), ParamsError> {
        fill(&mut self.max_cll, value)
    }

    pub fn set_max_fall(&mut self, value: u32) -> Result<(), ParamsError> {
        fill(&mut self.max_fall, value)
    }

    pub fn build(self) -> Result<ImageDescription, ParamsError> {
        let (Some(transfer), Some(primaries)) = (self.transfer, self.primaries) else {
            return Err(ParamsError::IncompleteSet);
        };
        let standard = ImageDescription::standard(transfer, primaries);
        let luminances = self.luminances.map_or(standard.luminances, |luminances| {
            swing(transfer, luminances)
        });
        let target = Target {
            primaries: self
                .mastering_primaries
                .unwrap_or(standard.target.primaries),
            luminance: self
                .mastering_luminance
                .filter(known_range)
                .unwrap_or(luminances.range()),
        };
        let light = ContentLight {
            max_cll: self.max_cll.filter(known),
            max_fall: self.max_fall.filter(known),
        };
        check_light(light)?;
        Ok(ImageDescription {
            luminances,
            target,
            light,
            ..standard
        })
    }
}

fn fill<T>(slot: &mut Option<T>, value: T) -> Result<(), ParamsError> {
    match slot {
        Some(_) => Err(ParamsError::AlreadySet),
        None => {
            *slot = Some(value);
            Ok(())
        }
    }
}

fn brighter(min: u32, nits: u32) -> bool {
    u64::from(nits) * u64::from(MIN_SCALE) > u64::from(min)
}

fn swing(transfer: TransferFunction, luminances: Luminances) -> Luminances {
    match transfer {
        TransferFunction::St2084Pq => Luminances {
            max: luminances.min / MIN_SCALE + PQ_SWING,
            ..luminances
        },
        TransferFunction::Srgb | TransferFunction::Gamma22 | TransferFunction::ExtLinear => {
            luminances
        }
    }
}

fn known(nits: &u32) -> bool {
    *nits != UNKNOWN
}

fn known_range(range: &Range) -> bool {
    known(&range.max)
}

fn check_light(light: ContentLight) -> Result<(), ParamsError> {
    match (light.max_cll, light.max_fall) {
        (Some(max_cll), Some(max_fall)) if max_fall > max_cll => Err(ParamsError::InvalidLuminance),
        _ => Ok(()),
    }
}
