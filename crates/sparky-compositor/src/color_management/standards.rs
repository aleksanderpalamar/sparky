use super::description::{Chromaticities, Luminances, NamedPrimaries, TransferFunction, Xy};

pub const PQ_SWING: u32 = 10_000;
const D65: Xy = xy(312_700, 329_000);
const DCI_WHITE: Xy = xy(314_000, 351_000);
const SRGB_LUMINANCES: Luminances = Luminances {
    min: 2_000,
    max: 80,
    reference: 80,
};
const PQ_LUMINANCES: Luminances = Luminances {
    min: 50,
    max: PQ_SWING,
    reference: 203,
};

impl NamedPrimaries {
    pub const fn chromaticities(self) -> Chromaticities {
        match self {
            Self::Srgb => gamut(
                xy(640_000, 330_000),
                xy(300_000, 600_000),
                xy(150_000, 60_000),
                D65,
            ),
            Self::Bt2020 => gamut(
                xy(708_000, 292_000),
                xy(170_000, 797_000),
                xy(131_000, 46_000),
                D65,
            ),
            Self::DisplayP3 => p3(D65),
            Self::DciP3 => p3(DCI_WHITE),
        }
    }
}

impl TransferFunction {
    pub const fn default_luminances(self) -> Luminances {
        match self {
            Self::St2084Pq => PQ_LUMINANCES,
            Self::Srgb | Self::Gamma22 | Self::ExtLinear => SRGB_LUMINANCES,
        }
    }
}

const fn p3(white: Xy) -> Chromaticities {
    gamut(
        xy(680_000, 320_000),
        xy(265_000, 690_000),
        xy(150_000, 60_000),
        white,
    )
}

const fn gamut(red: Xy, green: Xy, blue: Xy, white: Xy) -> Chromaticities {
    Chromaticities {
        red,
        green,
        blue,
        white,
    }
}

const fn xy(x: i32, y: i32) -> Xy {
    Xy { x, y }
}

#[cfg(test)]
mod tests {
    use super::super::description::{NamedPrimaries, TransferFunction, Xy};

    #[test]
    fn primarias_seguem_as_normas() {
        let bt2020 = NamedPrimaries::Bt2020.chromaticities();
        assert_eq!(
            bt2020.red,
            Xy {
                x: 708_000,
                y: 292_000
            }
        );
        assert_eq!(
            bt2020.green,
            Xy {
                x: 170_000,
                y: 797_000
            }
        );
        assert_eq!(
            bt2020.blue,
            Xy {
                x: 131_000,
                y: 46_000
            }
        );
        assert_eq!(
            bt2020.white,
            Xy {
                x: 312_700,
                y: 329_000
            }
        );
        let p3 = NamedPrimaries::DisplayP3.chromaticities();
        let dci = NamedPrimaries::DciP3.chromaticities();
        assert_eq!((p3.red, p3.green, p3.blue), (dci.red, dci.green, dci.blue));
        assert_eq!(
            dci.white,
            Xy {
                x: 314_000,
                y: 351_000
            }
        );
        assert_eq!(NamedPrimaries::Srgb.chromaticities().white, p3.white);
    }

    #[test]
    fn so_o_pq_muda_as_luminancias_padrao() {
        let pq = TransferFunction::St2084Pq.default_luminances();
        assert_eq!((pq.min, pq.max, pq.reference), (50, 10_000, 203));
        for transfer in [
            TransferFunction::Srgb,
            TransferFunction::Gamma22,
            TransferFunction::ExtLinear,
        ] {
            let sdr = transfer.default_luminances();
            assert_eq!((sdr.min, sdr.max, sdr.reference), (2_000, 80, 80));
        }
    }
}
