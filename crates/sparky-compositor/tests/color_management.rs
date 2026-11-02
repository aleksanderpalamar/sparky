#[path = "color_management/client.rs"]
mod client;
#[path = "support/compositor.rs"]
mod compositor;
#[path = "color_management/conversion.rs"]
mod conversion;
#[path = "color_management/creation.rs"]
mod creation;
#[path = "color_management/events.rs"]
mod events;
#[path = "color_management/fixture.rs"]
mod fixture;
#[path = "color_management/info.rs"]
mod info;
#[path = "support/ready.rs"]
mod ready;
#[path = "support/server.rs"]
mod server;
#[path = "color_management/surfaces.rs"]
mod surfaces;
#[path = "color_management/transition.rs"]
mod transition;

use client::Client;
use events::manager::{Feature, Primaries, RenderIntent, TransferFunction};
use fixture::Error;
use wayland_client::WEnum;

const SRGB: [i32; 8] = [
    640_000, 330_000, 300_000, 600_000, 150_000, 60_000, 312_700, 329_000,
];

#[test]
fn anuncia_versao_2_com_intencao_recursos_curvas_e_primarias() -> Result<(), Error> {
    let client = Client::connect()?;
    let global = client
        .globals
        .contents()
        .clone_list()
        .into_iter()
        .find(|global| global.interface == "wp_color_manager_v1");
    assert_eq!(global.map(|global| global.version), Some(2));
    let events = &client.events;
    assert_eq!(events.intents, [WEnum::Value(RenderIntent::Perceptual)]);
    assert_eq!(
        events.features,
        [
            Feature::Parametric,
            Feature::SetLuminances,
            Feature::SetMasteringDisplayPrimaries,
            Feature::ExtendedTargetVolume,
            Feature::WindowsScrgb,
        ]
        .map(WEnum::Value)
    );
    assert_eq!(
        events.transfers,
        [
            TransferFunction::CompoundPower24,
            TransferFunction::Gamma22,
            TransferFunction::ExtLinear,
            TransferFunction::St2084Pq,
        ]
        .map(WEnum::Value)
    );
    assert_eq!(
        events.primaries,
        [
            Primaries::Srgb,
            Primaries::Bt2020,
            Primaries::DisplayP3,
            Primaries::DciP3,
        ]
        .map(WEnum::Value)
    );
    assert_eq!(events.announced, 1);
    Ok(())
}

#[test]
fn monitor_sdr_descreve_srgb_com_todos_os_detalhes() -> Result<(), Error> {
    let mut client = Client::connect()?;
    let (image, identity) = client.output_image()?;
    assert_ne!(identity, 0);
    client.information(&image)?;
    let info = &client.events.info;
    assert_eq!(info.primaries, Some(SRGB));
    assert_eq!(info.primaries_named, Some(WEnum::Value(Primaries::Srgb)));
    assert_eq!(
        info.transfer,
        Some(WEnum::Value(TransferFunction::CompoundPower24))
    );
    assert_eq!(info.luminances, Some((2_000, 80, 80)));
    assert_eq!(info.target_primaries, Some(SRGB));
    assert_eq!(info.target_luminance, Some((2_000, 80)));
    assert!(info.done);
    let (_, de_novo) = client.output_image()?;
    assert_eq!(de_novo, identity);
    Ok(())
}

#[test]
fn versao_1_recebe_a_curva_srgb_antiga_no_anuncio_e_no_monitor() -> Result<(), Error> {
    let mut client = Client::with_version(1)?;
    let srgb = WEnum::Value(TransferFunction::Srgb);
    let transfers = &client.events.transfers;
    assert_eq!(transfers.first(), Some(&srgb));
    assert!(!transfers.contains(&WEnum::Value(TransferFunction::CompoundPower24)));
    let (image, _) = client.output_image()?;
    client.information(&image)?;
    assert_eq!(client.events.info.transfer, Some(srgb));
    Ok(())
}
