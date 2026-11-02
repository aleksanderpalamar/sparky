use super::client::Client;
use super::events::manager::{Primaries, RenderIntent, TransferFunction};
use super::fixture::Error;

#[test]
fn descricao_pq_converte_so_depois_do_commit_e_sai_ao_desfazer() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let qh = client.queue.handle();
    let surface = client.surface();
    let color = client.manager.get_surface(&surface, &qh, ());
    let creator = client.manager.create_parametric_creator(&qh, ());
    creator.set_tf_named(TransferFunction::St2084Pq);
    creator.set_primaries_named(Primaries::Bt2020);
    let image = creator.create(&qh, ());
    client.ready(&image)?;
    color.set_image_description(&image, RenderIntent::Perceptual);
    assert!(client.conversions()?.is_empty());
    surface.commit();
    assert_eq!(client.conversions()?, ["Pq"]);
    color.unset_image_description();
    surface.commit();
    assert!(client.conversions()?.is_empty());
    Ok(())
}

#[test]
fn descricao_do_monitor_sdr_nao_converte_e_scrgb_converte() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let qh = client.queue.handle();
    let monitor = client.surface();
    let color = client.manager.get_surface(&monitor, &qh, ());
    let (image, _) = client.output_image()?;
    color.set_image_description(&image, RenderIntent::Perceptual);
    monitor.commit();
    assert!(client.conversions()?.is_empty());
    let jogo = client.surface();
    let scrgb = client.manager.get_surface(&jogo, &qh, ());
    let linear = client.manager.create_windows_scrgb(&qh, ());
    client.ready(&linear)?;
    scrgb.set_image_description(&linear, RenderIntent::Perceptual);
    jogo.commit();
    assert_eq!(client.conversions()?, ["Linear"]);
    Ok(())
}
