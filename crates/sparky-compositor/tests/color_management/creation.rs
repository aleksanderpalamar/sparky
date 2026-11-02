use super::client::Client;
use super::events::{image, manager, params};
use super::fixture::Error;

const MANAGER_UNSUPPORTED_FEATURE: u32 = 0;
const NO_INFORMATION: u32 = 1;
const INCOMPLETE_SET: u32 = 0;
const ALREADY_SET: u32 = 1;
const UNSUPPORTED_FEATURE: u32 = 2;
const INVALID_TF: u32 = 3;
const INVALID_LUMINANCE: u32 = 5;

fn creator(client: &Client) -> params::WpImageDescriptionCreatorParamsV1 {
    client
        .manager
        .create_parametric_creator(&client.queue.handle(), ())
}

fn hdr10(client: &Client) -> image::WpImageDescriptionV1 {
    let creator = creator(client);
    creator.set_tf_named(manager::TransferFunction::St2084Pq);
    creator.set_primaries_named(manager::Primaries::Bt2020);
    creator.set_mastering_display_primaries(
        680_000, 320_000, 265_000, 690_000, 150_000, 60_000, 312_700, 329_000,
    );
    creator.set_mastering_luminance(50, 1_000);
    creator.set_max_cll(900);
    creator.set_max_fall(400);
    creator.create(&client.queue.handle(), ())
}

#[test]
fn descricoes_iguais_do_cliente_compartilham_a_identidade() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let primeira = hdr10(&client);
    let segunda = hdr10(&client);
    let identidade = client.ready(&primeira)?;
    assert_eq!(client.ready(&segunda)?, identidade);
    let (_, monitor) = client.output_image()?;
    assert_ne!(monitor, identidade);
    Ok(())
}

#[test]
fn descricao_criada_pelo_cliente_nao_tem_detalhes() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let image = hdr10(&client);
    client.ready(&image)?;
    image.get_information(&client.queue.handle(), ());
    assert_eq!(
        client.protocol_error(),
        Some((NO_INFORMATION, "wp_image_description_v1".into()))
    );
    Ok(())
}

#[test]
fn scrgb_do_windows_fica_pronto_sem_detalhes() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let image = client
        .manager
        .create_windows_scrgb(&client.queue.handle(), ());
    client.ready(&image)?;
    image.get_information(&client.queue.handle(), ());
    assert_eq!(
        client.protocol_error(),
        Some((NO_INFORMATION, "wp_image_description_v1".into()))
    );
    Ok(())
}

fn rejected(
    build: impl FnOnce(&params::WpImageDescriptionCreatorParamsV1, &Client),
) -> Result<Option<(u32, String)>, Error> {
    let mut client = Client::connect()?;
    let creator = creator(&client);
    build(&creator, &client);
    Ok(client.protocol_error())
}

fn erro(code: u32) -> Option<(u32, String)> {
    Some((code, "wp_image_description_creator_params_v1".into()))
}

#[test]
fn conjunto_sem_primarias_e_recusado_ao_criar() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let creator = creator(&client);
    creator.set_tf_named(manager::TransferFunction::CompoundPower24);
    creator.create(&client.queue.handle(), ());
    let error = client.protocol_error().map(|(code, _)| code);
    assert_eq!(error, Some(INCOMPLETE_SET));
    Ok(())
}

#[test]
fn curva_repetida_e_recusada() -> Result<(), Error> {
    let error = rejected(|creator, _| {
        creator.set_tf_named(manager::TransferFunction::CompoundPower24);
        creator.set_tf_named(manager::TransferFunction::Gamma22);
    })?;
    assert_eq!(error, erro(ALREADY_SET));
    Ok(())
}

#[test]
fn curva_nao_anunciada_e_recusada() -> Result<(), Error> {
    for curva in [
        manager::TransferFunction::Bt1886,
        manager::TransferFunction::Srgb,
    ] {
        let error = rejected(|creator, _| creator.set_tf_named(curva))?;
        assert_eq!(error, erro(INVALID_TF), "{curva:?}");
    }
    Ok(())
}

#[test]
fn versao_1_aceita_a_curva_srgb_antiga_e_recusa_a_nova() -> Result<(), Error> {
    let mut client = Client::with_version(1)?;
    let antiga = creator(&client);
    antiga.set_tf_named(manager::TransferFunction::Srgb);
    antiga.set_primaries_named(manager::Primaries::Srgb);
    let image = antiga.create(&client.queue.handle(), ());
    client.ready(&image)?;
    let nova = creator(&client);
    nova.set_tf_named(manager::TransferFunction::CompoundPower24);
    assert_eq!(client.protocol_error(), erro(INVALID_TF));
    Ok(())
}

#[test]
fn recursos_nao_anunciados_sao_recusados() -> Result<(), Error> {
    let potencia = rejected(|creator, _| creator.set_tf_power(22_000))?;
    assert_eq!(potencia, erro(UNSUPPORTED_FEATURE));
    let primarias = rejected(|creator, _| creator.set_primaries(1, 2, 3, 4, 5, 6, 7, 8))?;
    assert_eq!(primarias, erro(UNSUPPORTED_FEATURE));
    Ok(())
}

#[test]
fn luminancias_fora_de_ordem_sao_recusadas() -> Result<(), Error> {
    let error = rejected(|creator, _| creator.set_luminances(10_000, 1, 100))?;
    assert_eq!(error, erro(INVALID_LUMINANCE));
    Ok(())
}

#[test]
fn perfil_icc_nao_e_aceito() -> Result<(), Error> {
    let mut client = Client::connect()?;
    client
        .manager
        .create_icc_creator(&client.queue.handle(), ());
    assert_eq!(
        client.protocol_error(),
        Some((MANAGER_UNSUPPORTED_FEATURE, "wp_color_manager_v1".into()))
    );
    Ok(())
}

#[test]
fn metadados_desconhecidos_da_mesa_e_do_firefox_nao_derrubam_o_cliente() -> Result<(), Error> {
    let mut client = Client::with_version(1)?;
    let qh = client.queue.handle();
    let mesa = creator(&client);
    mesa.set_primaries_named(manager::Primaries::Bt2020);
    mesa.set_tf_named(manager::TransferFunction::St2084Pq);
    mesa.set_max_cll(0);
    mesa.set_max_fall(0);
    mesa.set_mastering_display_primaries(
        708_000, 292_000, 170_000, 797_000, 131_000, 46_000, 312_700, 329_000,
    );
    let video = mesa.create(&qh, ());
    let firefox = creator(&client);
    firefox.set_primaries_named(manager::Primaries::Bt2020);
    firefox.set_tf_named(manager::TransferFunction::St2084Pq);
    firefox.set_mastering_luminance(50, 0);
    firefox.set_max_cll(1_200);
    let pagina = firefox.create(&qh, ());
    client.ready(&video)?;
    client.ready(&pagina)?;
    assert_eq!(client.protocol_error(), None);
    Ok(())
}
