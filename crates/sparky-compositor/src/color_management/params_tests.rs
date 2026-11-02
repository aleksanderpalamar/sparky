use super::description::{
    ContentLight, ImageDescription, Luminances, NamedPrimaries, Range, Target, TransferFunction,
};
use super::params::{Params, ParamsError};

fn named(transfer: TransferFunction, primaries: NamedPrimaries) -> Result<Params, ParamsError> {
    let mut params = Params::default();
    params.set_transfer(transfer)?;
    params.set_primaries(primaries)?;
    Ok(params)
}

#[test]
fn sem_curva_ou_primarias_o_conjunto_esta_incompleto() -> Result<(), ParamsError> {
    assert_eq!(Params::default().build(), Err(ParamsError::IncompleteSet));
    let mut curva = Params::default();
    curva.set_transfer(TransferFunction::Srgb)?;
    assert_eq!(curva.build(), Err(ParamsError::IncompleteSet));
    let mut primarias = Params::default();
    primarias.set_primaries(NamedPrimaries::Srgb)?;
    assert_eq!(primarias.build(), Err(ParamsError::IncompleteSet));
    Ok(())
}

#[test]
fn curva_e_primarias_bastam_e_usam_as_luminancias_padrao() -> Result<(), ParamsError> {
    let description = named(TransferFunction::Srgb, NamedPrimaries::Srgb)?.build()?;
    let luminances = Luminances {
        min: 2_000,
        max: 80,
        reference: 80,
    };
    assert_eq!(
        description,
        ImageDescription {
            transfer: TransferFunction::Srgb,
            primaries: NamedPrimaries::Srgb,
            luminances,
            target: Target {
                primaries: NamedPrimaries::Srgb.chromaticities(),
                luminance: Range {
                    min: 2_000,
                    max: 80
                },
            },
            light: ContentLight::default(),
        }
    );
    Ok(())
}

#[test]
fn pq_assume_as_luminancias_do_hdr10() -> Result<(), ParamsError> {
    let description = named(TransferFunction::St2084Pq, NamedPrimaries::Bt2020)?.build()?;
    let esperado = Luminances {
        min: 50,
        max: 10_000,
        reference: 203,
    };
    assert_eq!(description.luminances, esperado);
    assert_eq!(
        description.target.luminance,
        Range {
            min: 50,
            max: 10_000
        }
    );
    Ok(())
}

#[test]
fn cada_propriedade_so_pode_ser_definida_uma_vez() -> Result<(), ParamsError> {
    let mut params = named(TransferFunction::Gamma22, NamedPrimaries::DisplayP3)?;
    let repetida = Err(ParamsError::AlreadySet);
    assert_eq!(params.set_transfer(TransferFunction::Srgb), repetida);
    assert_eq!(params.set_primaries(NamedPrimaries::Srgb), repetida);
    let luminances = Luminances {
        min: 2_000,
        max: 80,
        reference: 80,
    };
    params.set_luminances(luminances)?;
    assert_eq!(params.set_luminances(luminances), repetida);
    params.set_mastering_primaries(NamedPrimaries::Srgb.chromaticities())?;
    assert_eq!(
        params.set_mastering_primaries(NamedPrimaries::Srgb.chromaticities()),
        repetida
    );
    params.set_mastering_luminance(Range { min: 0, max: 80 })?;
    assert_eq!(
        params.set_mastering_luminance(Range { min: 0, max: 80 }),
        repetida
    );
    params.set_max_cll(70)?;
    assert_eq!(params.set_max_cll(70), repetida);
    params.set_max_fall(60)?;
    assert_eq!(params.set_max_fall(60), repetida);
    Ok(())
}

#[test]
fn maximo_ou_referencia_ate_o_minimo_sao_invalidos() {
    let invalida = Err(ParamsError::InvalidLuminance);
    let teto = Luminances {
        min: 10_000,
        max: 1,
        reference: 100,
    };
    assert_eq!(Params::default().set_luminances(teto), invalida);
    let referencia = Luminances {
        min: 10_000,
        max: 100,
        reference: 1,
    };
    assert_eq!(Params::default().set_luminances(referencia), invalida);
    let mastering = Range {
        min: 1_000_000,
        max: 100,
    };
    assert_eq!(
        Params::default().set_mastering_luminance(mastering),
        invalida
    );
}

#[test]
fn com_pq_o_maximo_e_o_minimo_mais_dez_mil() -> Result<(), ParamsError> {
    let mut params = named(TransferFunction::St2084Pq, NamedPrimaries::Bt2020)?;
    params.set_luminances(Luminances {
        min: 50_000,
        max: 400,
        reference: 203,
    })?;
    let description = params.build()?;
    assert_eq!(description.luminances.max, 10_005);
    assert_eq!(
        description.target.luminance,
        Range {
            min: 50_000,
            max: 10_005
        }
    );
    Ok(())
}

#[test]
fn mastering_define_o_volume_alvo() -> Result<(), ParamsError> {
    let mut params = named(TransferFunction::St2084Pq, NamedPrimaries::Bt2020)?;
    params.set_mastering_primaries(NamedPrimaries::DisplayP3.chromaticities())?;
    params.set_mastering_luminance(Range {
        min: 50,
        max: 1_000,
    })?;
    params.set_max_cll(900)?;
    params.set_max_fall(400)?;
    let description = params.build()?;
    assert_eq!(
        description.target,
        Target {
            primaries: NamedPrimaries::DisplayP3.chromaticities(),
            luminance: Range {
                min: 50,
                max: 1_000
            },
        }
    );
    assert_eq!(
        description.light,
        ContentLight {
            max_cll: Some(900),
            max_fall: Some(400),
        }
    );
    Ok(())
}

#[test]
fn media_de_quadro_acima_do_pico_e_invalida() -> Result<(), ParamsError> {
    let mut params = named(TransferFunction::St2084Pq, NamedPrimaries::Bt2020)?;
    params.set_max_cll(500)?;
    params.set_max_fall(600)?;
    assert_eq!(params.build(), Err(ParamsError::InvalidLuminance));
    Ok(())
}
