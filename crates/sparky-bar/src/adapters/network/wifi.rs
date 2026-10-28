use super::{profiles::Profile, proxy};
use crate::domain::network::{Security, WifiNetwork};
use zbus::{Connection, zvariant::OwnedObjectPath};

pub async fn read(
    connection: &Connection,
    device: &str,
    profiles: &[Profile],
) -> zbus::Result<Vec<WifiNetwork>> {
    let wifi = proxy(
        connection,
        device,
        "org.freedesktop.NetworkManager.Device.Wireless",
    )
    .await?;
    let active: OwnedObjectPath = wifi.get_property("ActiveAccessPoint").await?;
    let points: Vec<OwnedObjectPath> = wifi.call("GetAllAccessPoints", &()).await?;
    let mut networks = Vec::new();
    for path in points {
        let ap = proxy(
            connection,
            path.as_str(),
            "org.freedesktop.NetworkManager.AccessPoint",
        )
        .await?;
        let ssid: Vec<u8> = ap.get_property("Ssid").await?;
        let flags = ap.get_property("Flags").await?;
        let wpa = ap.get_property("WpaFlags").await?;
        let rsn = ap.get_property("RsnFlags").await?;
        let security = Security::from_flags(flags, wpa, rsn);
        let profile = profiles
            .iter()
            .find(|profile| profile.ssid == ssid && profile.security.matches_flags(flags, wpa, rsn))
            .map(|profile| profile.path.clone());
        networks.push(WifiNetwork {
            path: path.to_string(),
            device: device.to_owned(),
            ssid,
            strength: ap.get_property("Strength").await?,
            security,
            profile,
            active: active == path,
        });
    }
    Ok(networks)
}
