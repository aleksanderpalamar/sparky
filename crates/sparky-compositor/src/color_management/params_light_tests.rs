use super::description::{ContentLight, NamedPrimaries, Range, TransferFunction};
use super::params::{Params, ParamsError};

fn hdr10() -> Result<Params, ParamsError> {
    let mut params = Params::default();
    params.set_transfer(TransferFunction::St2084Pq)?;
    params.set_primaries(NamedPrimaries::Bt2020)?;
    Ok(params)
}

#[test]
fn luz_zerada_e_desconhecida_como_a_mesa_manda() -> Result<(), ParamsError> {
    let mut params = hdr10()?;
    params.set_max_cll(0)?;
    params.set_max_fall(0)?;
    let description = params.build()?;
    assert_eq!(description.light, ContentLight::default());
    Ok(())
}

#[test]
fn luz_zerada_continua_contando_como_definida() -> Result<(), ParamsError> {
    let mut params = hdr10()?;
    params.set_max_cll(0)?;
    params.set_max_fall(0)?;
    assert_eq!(params.set_max_cll(0), Err(ParamsError::AlreadySet));
    assert_eq!(params.set_max_fall(0), Err(ParamsError::AlreadySet));
    Ok(())
}

#[test]
fn pico_de_masterizacao_zerado_e_desconhecido_como_o_firefox_manda() -> Result<(), ParamsError> {
    let mut params = hdr10()?;
    params.set_mastering_luminance(Range { min: 50, max: 0 })?;
    assert_eq!(
        params.set_mastering_luminance(Range { min: 50, max: 0 }),
        Err(ParamsError::AlreadySet)
    );
    let description = params.build()?;
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
fn luz_acima_do_pico_de_masterizacao_e_aceita_como_na_versao_2() -> Result<(), ParamsError> {
    let mut params = hdr10()?;
    params.set_mastering_luminance(Range {
        min: 50,
        max: 1_000,
    })?;
    params.set_max_cll(1_200)?;
    params.set_max_fall(400)?;
    let description = params.build()?;
    assert_eq!(description.light.max_cll, Some(1_200));
    assert_eq!(description.light.max_fall, Some(400));
    Ok(())
}
