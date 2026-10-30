use super::{ROOT, SERVICE, proxy};
use crate::domain::network::{DeviceKind, DeviceState, NetworkDevice, NetworkState};
use zbus::{Connection, zvariant::OwnedObjectPath};

pub async fn read(connection: &Connection) -> zbus::Result<NetworkState> {
    let root = proxy(connection, ROOT, SERVICE).await?;
    let paths: Vec<OwnedObjectPath> = root.call("GetDevices", &()).await?;
    let mut state = NetworkState {
        enabled: root.get_property("NetworkingEnabled").await?,
        wireless: root.get_property("WirelessEnabled").await?,
        hardware_wireless: root.get_property("WirelessHardwareEnabled").await?,
        connectivity: root.get_property("Connectivity").await?,
        ..Default::default()
    };
    let profiles = super::profiles::read(connection).await?;
    for path in paths {
        let Some(device) = read_device(connection, path.as_str()).await? else {
            continue;
        };
        if device.kind == DeviceKind::Wifi {
            state
                .networks
                .extend(super::wifi::read(connection, &device.path, &profiles).await?);
        }
        state.devices.push(device);
    }
    state.networks.sort_by(|a, b| {
        b.active
            .cmp(&a.active)
            .then(b.strength.cmp(&a.strength))
            .then(a.ssid.cmp(&b.ssid))
    });
    let mut seen = std::collections::HashSet::new();
    state.networks.retain(|network| {
        !network.ssid.is_empty()
            && seen.insert((
                network.device.clone(),
                network.ssid.clone(),
                network.security,
            ))
    });
    Ok(state)
}

async fn read_device(connection: &Connection, path: &str) -> zbus::Result<Option<NetworkDevice>> {
    let device = proxy(connection, path, "org.freedesktop.NetworkManager.Device").await?;
    let Some(kind) = DeviceKind::from_nm(device.get_property("DeviceType").await?) else {
        return Ok(None);
    };
    let active: OwnedObjectPath = device.get_property("ActiveConnection").await?;
    let ip: OwnedObjectPath = device.get_property("Ip4Config").await?;
    let mut result = NetworkDevice {
        path: path.to_owned(),
        interface: device.get_property("Interface").await?,
        name: None,
        speed: None,
        kind,
        state: DeviceState::from_nm(device.get_property("State").await?),
        addresses: Vec::new(),
        gateway: None,
        active_connection: (active.as_str() != "/").then(|| active.to_string()),
    };
    if active.as_str() != "/" {
        result.name = read_connection_name(connection, active.as_str()).await;
    }
    if result.kind == DeviceKind::Ethernet {
        result.speed = read_wired_speed(connection, path).await;
    }
    if ip.as_str() == "/" {
        return Ok(Some(result));
    }
    let config = proxy(
        connection,
        ip.as_str(),
        "org.freedesktop.NetworkManager.IP4Config",
    )
    .await?;
    let addresses: Vec<std::collections::HashMap<String, zbus::zvariant::OwnedValue>> =
        config.get_property("AddressData").await?;
    result.addresses = addresses
        .iter()
        .filter_map(|entry| {
            let address = <&str>::try_from(entry.get("address")?).ok()?;
            let prefix = u32::try_from(entry.get("prefix")?).ok()?;
            Some(format!("{address}/{prefix}"))
        })
        .collect();
    let gateway: String = config.get_property("Gateway").await?;
    result.gateway = (!gateway.is_empty()).then_some(gateway);
    Ok(Some(result))
}

async fn read_connection_name(connection: &Connection, path: &str) -> Option<String> {
    let active = proxy(
        connection,
        path,
        "org.freedesktop.NetworkManager.Connection.Active",
    )
    .await
    .ok()?;
    active.get_property("Id").await.ok()
}

async fn read_wired_speed(connection: &Connection, path: &str) -> Option<u32> {
    let wired = proxy(
        connection,
        path,
        "org.freedesktop.NetworkManager.Device.Wired",
    )
    .await
    .ok()?;
    wired.get_property("Speed").await.ok()
}
