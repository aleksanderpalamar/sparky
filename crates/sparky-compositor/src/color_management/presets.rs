use super::description::{
    ImageDescription, Luminances, NamedPrimaries, Range, Target, TransferFunction,
};
use super::standards::PQ_SWING;

const SCRGB_UNIT_NITS: u32 = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputColor {
    Sdr,
    Hdr(Range),
}

pub fn output(color: OutputColor) -> ImageDescription {
    match color {
        OutputColor::Sdr => {
            ImageDescription::standard(TransferFunction::Srgb, NamedPrimaries::Srgb)
        }
        OutputColor::Hdr(display) => {
            let hdr10 =
                ImageDescription::standard(TransferFunction::St2084Pq, NamedPrimaries::Bt2020);
            ImageDescription {
                target: Target {
                    luminance: display,
                    ..hdr10.target
                },
                ..hdr10
            }
        }
    }
}

pub fn windows_scrgb() -> ImageDescription {
    let pq = TransferFunction::St2084Pq.default_luminances();
    ImageDescription {
        luminances: Luminances {
            min: 0,
            max: SCRGB_UNIT_NITS,
            reference: pq.reference,
        },
        target: Target {
            primaries: NamedPrimaries::Bt2020.chromaticities(),
            luminance: Range {
                min: 0,
                max: PQ_SWING,
            },
        },
        ..ImageDescription::standard(TransferFunction::ExtLinear, NamedPrimaries::Srgb)
    }
}

#[cfg(test)]
mod tests {
    use super::{OutputColor, output, windows_scrgb};
    use crate::color_management::description::{
        ImageDescription, Luminances, NamedPrimaries, Range, Target, TransferFunction,
    };

    #[test]
    fn tela_sdr_espera_srgb() {
        assert_eq!(
            output(OutputColor::Sdr),
            ImageDescription::standard(TransferFunction::Srgb, NamedPrimaries::Srgb)
        );
    }

    #[test]
    fn tela_hdr_espera_pq_bt2020_ate_o_brilho_da_tela() {
        let tela = Range { min: 50, max: 600 };
        let description = output(OutputColor::Hdr(tela));
        assert_eq!(description.transfer, TransferFunction::St2084Pq);
        assert_eq!(description.primaries, NamedPrimaries::Bt2020);
        let pq = Luminances {
            min: 50,
            max: 10_000,
            reference: 203,
        };
        assert_eq!(description.luminances, pq);
        assert_eq!(
            description.target,
            Target {
                primaries: NamedPrimaries::Bt2020.chromaticities(),
                luminance: tela,
            }
        );
    }

    #[test]
    fn scrgb_do_windows_e_linear_com_branco_em_203_nits() {
        let description = windows_scrgb();
        assert_eq!(description.transfer, TransferFunction::ExtLinear);
        assert_eq!(description.primaries, NamedPrimaries::Srgb);
        let escala = Luminances {
            min: 0,
            max: 80,
            reference: 203,
        };
        assert_eq!(description.luminances, escala);
        assert_eq!(
            description.target,
            Target {
                primaries: NamedPrimaries::Bt2020.chromaticities(),
                luminance: Range {
                    min: 0,
                    max: 10_000
                },
            }
        );
    }
}
