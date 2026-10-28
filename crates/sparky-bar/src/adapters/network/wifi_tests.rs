use super::{profiles, wifi};
use crate::{
    adapters::test_bus::Bus,
    app::panel_flow_tests::wifi as network,
    domain::network::{Security, WifiNetwork},
};
use zbus::zvariant::OwnedObjectPath;

struct Wireless;

#[zbus::interface(name = "org.freedesktop.NetworkManager.Device.Wireless")]
impl Wireless {
    #[zbus(property)]
    fn active_access_point(&self) -> OwnedObjectPath {
        "/".try_into().unwrap()
    }

    fn get_all_access_points(&self) -> Vec<OwnedObjectPath> {
        vec!["/ap/1".try_into().unwrap()]
    }
}

struct AccessPoint {
    flags: u32,
    wpa: u32,
    rsn: u32,
}

#[zbus::interface(name = "org.freedesktop.NetworkManager.AccessPoint")]
impl AccessPoint {
    #[zbus(property)]
    fn ssid(&self) -> Vec<u8> {
        network().ssid
    }

    #[zbus(property)]
    fn flags(&self) -> u32 {
        self.flags
    }

    #[zbus(property)]
    fn wpa_flags(&self) -> u32 {
        self.wpa
    }

    #[zbus(property)]
    fn rsn_flags(&self) -> u32 {
        self.rsn
    }

    #[zbus(property)]
    fn strength(&self) -> u8 {
        80
    }
}

fn profile(security: Security) -> profiles::Profile {
    let mut network = network();
    network.security = security;
    let settings = profiles::create(&network, "12345678", true).unwrap();
    profiles::Profile {
        path: "/profile/saved".to_owned(),
        ssid: network.ssid,
        security: profiles::security(&settings),
    }
}

async fn read(flags: u32, wpa: u32, rsn: u32, profiles: &[profiles::Profile]) -> WifiNetwork {
    let bus = Bus::start();
    let nm = bus.connection().await;
    nm.object_server().at("/wifi/1", Wireless).await.unwrap();
    nm.object_server()
        .at("/ap/1", AccessPoint { flags, wpa, rsn })
        .await
        .unwrap();
    nm.request_name(super::SERVICE).await.unwrap();
    let client = bus.connection().await;
    let networks = wifi::read(&client, "/wifi/1", profiles).await.unwrap();
    assert_eq!(networks.len(), 1);
    networks.into_iter().next().unwrap()
}

#[tokio::test]
async fn rede_mista_reutiliza_perfil_wpa3_salvo() {
    for (wpa, rsn) in [(0, 0x500), (0x100, 0x400)] {
        let network = read(1, wpa, rsn, &[profile(Security::Sae)]).await;
        assert_eq!(network.profile.as_deref(), Some("/profile/saved"));
        assert_eq!(network.security, Security::Personal);
        assert!(!network.active);
        let mut bar = crate::app::state::Bar::new(
            sparky_theme::Skin::default(),
            std::sync::Arc::new(crate::ports::fake::FakeWorkspaces::default()),
            chrono::Local::now().naive_local(),
        );
        let _ = crate::app::network::choose(&mut bar, network);
        assert!(bar.panels.wifi.is_none());
    }
}

#[tokio::test]
async fn rede_mista_preserva_perfil_wpa2_salvo() {
    for (wpa, rsn) in [(0, 0x500), (0x100, 0x400)] {
        let network = read(1, wpa, rsn, &[profile(Security::Personal)]).await;
        assert_eq!(network.profile.as_deref(), Some("/profile/saved"));
    }
}

#[tokio::test]
async fn rede_recusa_perfis_incompativeis_ou_com_outro_ssid() {
    for (flags, rsn, security) in [
        (1, 0x100, Security::Sae),
        (1, 0x400, Security::Personal),
        (1, 0x500, Security::Open),
        (0, 0, Security::Personal),
    ] {
        assert!(
            read(flags, 0, rsn, &[profile(security)])
                .await
                .profile
                .is_none()
        );
    }
    let mut saved = profile(Security::Sae);
    saved.ssid.push(0xff);
    assert!(read(1, 0, 0x500, &[saved]).await.profile.is_none());
}
