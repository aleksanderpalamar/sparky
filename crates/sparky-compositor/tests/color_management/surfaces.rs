use super::client::Client;
use super::events::manager::RenderIntent;
use super::fixture::Error;

const SURFACE_EXISTS: u32 = 1;
const RENDER_INTENT: u32 = 0;
const INERT: u32 = 2;

#[test]
fn descricao_pronta_vale_no_commit_e_a_extensao_pode_ser_refeita() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let qh = client.queue.handle();
    let surface = client.surface();
    let color = client.manager.get_surface(&surface, &qh, ());
    let image = client.manager.create_windows_scrgb(&qh, ());
    client.ready(&image)?;
    color.set_image_description(&image, RenderIntent::Perceptual);
    image.destroy();
    surface.commit();
    color.unset_image_description();
    surface.commit();
    color.destroy();
    let _ = client.manager.get_surface(&surface, &qh, ());
    assert_eq!(client.protocol_error(), None);
    Ok(())
}

#[test]
fn segunda_extensao_da_mesma_superficie_e_recusada() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let qh = client.queue.handle();
    let surface = client.surface();
    let _primeira = client.manager.get_surface(&surface, &qh, ());
    let _segunda = client.manager.get_surface(&surface, &qh, ());
    assert_eq!(
        client.protocol_error(),
        Some((SURFACE_EXISTS, "wp_color_manager_v1".into()))
    );
    Ok(())
}

#[test]
fn intencao_nao_anunciada_e_recusada() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let qh = client.queue.handle();
    let surface = client.surface();
    let color = client.manager.get_surface(&surface, &qh, ());
    let (image, _) = client.output_image()?;
    color.set_image_description(&image, RenderIntent::Relative);
    assert_eq!(
        client.protocol_error(),
        Some((RENDER_INTENT, "wp_color_management_surface_v1".into()))
    );
    Ok(())
}

#[test]
fn superficie_destruida_deixa_a_extensao_inerte() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let qh = client.queue.handle();
    let surface = client.surface();
    let color = client.manager.get_surface(&surface, &qh, ());
    surface.destroy();
    color.unset_image_description();
    assert_eq!(
        client.protocol_error(),
        Some((INERT, "wp_color_management_surface_v1".into()))
    );
    Ok(())
}
