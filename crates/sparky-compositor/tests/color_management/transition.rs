use super::client::Client;
use super::events::manager::{Primaries, TransferFunction};
use super::fixture::Error;
use wayland_client::WEnum;

const BT2020: [i32; 8] = [
    708_000, 292_000, 170_000, 797_000, 131_000, 46_000, 312_700, 329_000,
];

#[test]
fn monitor_em_hdr_avisa_os_clientes_e_descreve_pq_bt2020() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let qh = client.queue.handle();
    let output = client.manager.get_output(&client.output, &qh, ());
    let surface = client.surface();
    let feedback = client.manager.get_surface_feedback(&surface, &qh, ());
    let antes = output.get_image_description(&qh, ());
    let sdr = client.ready(&antes)?;
    let preferida = feedback.get_preferred(&qh, ());
    assert_eq!(client.ready(&preferida)?, sdr);
    let feitos = client.events.output_done;
    client.signal("hdr")?;
    client.await_event(|events| events.output_changed == 1 && events.preferred.len() == 1)?;
    assert!(client.events.output_done > feitos);
    let hdr = client.events.preferred[0];
    assert_ne!(hdr, sdr);
    let depois = output.get_image_description(&qh, ());
    assert_eq!(client.ready(&depois)?, hdr);
    client.information(&depois)?;
    let info = &client.events.info;
    assert_eq!(info.primaries, Some(BT2020));
    assert_eq!(info.primaries_named, Some(WEnum::Value(Primaries::Bt2020)));
    assert_eq!(
        info.transfer,
        Some(WEnum::Value(TransferFunction::St2084Pq))
    );
    assert_eq!(info.luminances, Some((50, 10_000, 203)));
    assert_eq!(info.target_primaries, Some(BT2020));
    assert_eq!(info.target_luminance, Some((50, 600)));
    let parametrica = feedback.get_preferred_parametric(&qh, ());
    assert_eq!(client.ready(&parametrica)?, hdr);
    client.signal("sdr")?;
    client.await_event(|events| events.output_changed == 2 && events.preferred.len() == 2)?;
    assert_eq!(client.events.preferred[1], sdr);
    Ok(())
}

#[test]
fn cliente_da_versao_1_recebe_os_eventos_da_versao_1() -> Result<(), Error> {
    let mut client = Client::with_version(1)?;
    let qh = client.queue.handle();
    let surface = client.surface();
    let feedback = client.manager.get_surface_feedback(&surface, &qh, ());
    let preferida = feedback.get_preferred(&qh, ());
    let sdr = client.ready(&preferida)?;
    client.signal("hdr")?;
    client.await_event(|events| events.preferred.len() == 1)?;
    assert_ne!(client.events.preferred[0], sdr);
    Ok(())
}
