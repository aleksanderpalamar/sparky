use super::curves;
use super::description::{ImageDescription, NamedPrimaries, TransferFunction};
use super::gamut::{self, Matrix};
use super::standards::PQ_SWING;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decode {
    Srgb,
    Gamma22,
    Linear,
    Pq,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Conversion {
    pub decode: Decode,
    pub primaries: Matrix,
    pub scale: f64,
}

impl Conversion {
    pub fn convert(&self, rgb: [f64; 3]) -> [f64; 3] {
        let light = rgb.map(|channel| decode(self.decode, channel) * self.scale);
        gamut::apply(&self.primaries, light).map(curves::linear_to_srgb)
    }
}

fn decode(decode: Decode, value: f64) -> f64 {
    match decode {
        Decode::Srgb => curves::srgb_to_linear(value),
        Decode::Gamma22 => curves::gamma22_to_linear(value),
        Decode::Linear => value,
        Decode::Pq => curves::pq_to_nits(value) / curves::PEAK_NITS,
    }
}

pub fn conversion(description: &ImageDescription) -> Option<Conversion> {
    let luminances = description.luminances;
    let native = description.transfer == TransferFunction::Srgb
        && description.primaries == NamedPrimaries::Srgb
        && luminances.max == luminances.reference;
    if native {
        return None;
    }
    let (decode, top) = match description.transfer {
        TransferFunction::Srgb => (Decode::Srgb, luminances.max),
        TransferFunction::Gamma22 => (Decode::Gamma22, luminances.max),
        TransferFunction::ExtLinear => (Decode::Linear, luminances.max),
        TransferFunction::St2084Pq => (Decode::Pq, PQ_SWING),
    };
    Some(Conversion {
        decode,
        primaries: gamut::to_bt709(description.primaries),
        scale: f64::from(top) / f64::from(luminances.reference.max(1)),
    })
}

#[cfg(test)]
mod tests {
    use super::{Decode, conversion};
    use crate::color_management::curves::{linear_to_srgb, pq};
    use crate::color_management::description::{
        ImageDescription, Luminances, NamedPrimaries, TransferFunction,
    };
    use crate::color_management::gamut;
    use crate::color_management::presets::windows_scrgb;

    fn perto(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn proximo(a: [f64; 3], b: [f64; 3]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4)
    }

    fn hdr10() -> Result<super::Conversion, &'static str> {
        let description =
            ImageDescription::standard(TransferFunction::St2084Pq, NamedPrimaries::Bt2020);
        conversion(&description).ok_or("PQ precisa de conversão")
    }

    #[test]
    fn branco_de_referencia_do_pq_vira_o_branco_da_composicao() -> Result<(), &'static str> {
        let sinal = pq(203.0);
        let branco = hdr10()?.convert([sinal; 3]);
        assert!(proximo(branco, [1.0; 3]), "{branco:?}");
        Ok(())
    }

    #[test]
    fn mil_nits_passam_do_branco_na_proporcao_certa() -> Result<(), &'static str> {
        let brilho = hdr10()?.convert([pq(1_000.0); 3]);
        let esperado = linear_to_srgb(1_000.0 / 203.0);
        assert!(proximo(brilho, [esperado; 3]), "{brilho:?}");
        Ok(())
    }

    #[test]
    fn vermelho_bt2020_sai_do_srgb_com_canais_negativos() -> Result<(), &'static str> {
        let vermelho = hdr10()?.convert([pq(203.0), pq(0.0), pq(0.0)]);
        assert!(vermelho[0] > 1.0, "{vermelho:?}");
        assert!(vermelho[1] < 0.0 && vermelho[2] < 0.0, "{vermelho:?}");
        Ok(())
    }

    #[test]
    fn scrgb_2_5375_vira_branco_e_1_vira_80_nits() -> Result<(), &'static str> {
        let scrgb = conversion(&windows_scrgb()).ok_or("scRGB precisa de conversão")?;
        assert!(proximo(scrgb.convert([2.5375; 3]), [1.0; 3]));
        let oitenta = linear_to_srgb(80.0 / 203.0);
        assert!(proximo(scrgb.convert([1.0; 3]), [oitenta; 3]));
        Ok(())
    }

    #[test]
    fn gama_2_2_vira_srgb_com_a_mesma_luz() -> Result<(), &'static str> {
        let video = ImageDescription::standard(TransferFunction::Gamma22, NamedPrimaries::Srgb);
        let gama = conversion(&video).ok_or("gama 2.2 precisa de conversão")?;
        let esperado = linear_to_srgb(0.5_f64.powf(2.2));
        assert!(proximo(gama.convert([0.5; 3]), [esperado; 3]));
        Ok(())
    }

    #[test]
    fn srgb_padrao_dispensa_conversao() {
        let srgb = ImageDescription::standard(TransferFunction::Srgb, NamedPrimaries::Srgb);
        assert_eq!(conversion(&srgb), None);
    }

    #[test]
    fn srgb_com_branco_abaixo_do_maximo_fica_mais_claro() {
        let base = ImageDescription::standard(TransferFunction::Srgb, NamedPrimaries::Srgb);
        let claro = ImageDescription {
            luminances: Luminances {
                min: 2_000,
                max: 406,
                reference: 203,
            },
            ..base
        };
        let resultado = conversion(&claro);
        assert_eq!(resultado.map(|item| item.decode), Some(Decode::Srgb));
        assert!(resultado.is_some_and(|item| perto(item.scale, 2.0)));
    }

    #[test]
    fn pq_usa_os_dez_mil_nits_sobre_o_branco_de_referencia() -> Result<(), &'static str> {
        let hdr10 = ImageDescription::standard(TransferFunction::St2084Pq, NamedPrimaries::Bt2020);
        let Some(resultado) = conversion(&hdr10) else {
            return Err("PQ precisa de conversão");
        };
        assert_eq!(resultado.decode, Decode::Pq);
        assert!(perto(resultado.scale, 10_000.0 / 203.0));
        assert_eq!(resultado.primaries, gamut::to_bt709(NamedPrimaries::Bt2020));
        Ok(())
    }

    #[test]
    fn scrgb_tem_o_branco_de_referencia_em_2_5375() -> Result<(), &'static str> {
        let Some(resultado) = conversion(&windows_scrgb()) else {
            return Err("scRGB precisa de conversão");
        };
        assert_eq!(resultado.decode, Decode::Linear);
        assert!(perto(resultado.scale * 2.5375, 1.0));
        Ok(())
    }

    #[test]
    fn gama_2_2_troca_so_a_curva() -> Result<(), &'static str> {
        let video = ImageDescription::standard(TransferFunction::Gamma22, NamedPrimaries::Srgb);
        let Some(resultado) = conversion(&video) else {
            return Err("gama 2.2 precisa de conversão");
        };
        assert_eq!(resultado.decode, Decode::Gamma22);
        assert!(perto(resultado.scale, 1.0));
        assert_eq!(resultado.primaries, gamut::to_bt709(NamedPrimaries::Srgb));
        Ok(())
    }
}
