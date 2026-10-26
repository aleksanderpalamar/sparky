use std::collections::HashMap;

use zbus::fdo::ManagedObjects;
use zbus::zvariant::OwnedValue;

use crate::domain::bluetooth::{Adapter, Bluetooth};

pub const ADAPTER: &str = "org.bluez.Adapter1";
pub const DEVICE: &str = "org.bluez.Device1";

type Properties = HashMap<String, OwnedValue>;

pub fn bluetooth(objects: &ManagedObjects) -> Option<Bluetooth> {
    let mut adapters: Vec<_> = objects
        .iter()
        .filter_map(|(path, interfaces)| {
            let properties = interfaces.get(ADAPTER)?;
            let powered = flag(properties, "Powered");
            let mut connected: Vec<_> = objects
                .values()
                .filter_map(|interfaces| {
                    let device = interfaces.get(DEVICE)?;
                    if !powered
                        || !flag(device, "Connected")
                        || text(device, "Adapter").as_deref() != Some(path.as_str())
                    {
                        return None;
                    }
                    text(device, "Alias").or_else(|| text(device, "Name"))
                })
                .collect();
            connected.sort();
            let adapter = Adapter {
                name: text(properties, "Alias")
                    .or_else(|| text(properties, "Name"))
                    .unwrap_or_else(|| "Adaptador Bluetooth".to_owned()),
                powered,
                connected,
            };
            Some((path.as_str(), adapter))
        })
        .collect();
    adapters.sort_by_key(|(path, _)| *path);
    (!adapters.is_empty())
        .then(|| Bluetooth::Ready(adapters.into_iter().map(|(_, adapter)| adapter).collect()))
}

fn flag(properties: &Properties, key: &str) -> bool {
    properties
        .get(key)
        .and_then(|value| bool::try_from(value).ok())
        .unwrap_or(false)
}

fn text(properties: &Properties, key: &str) -> Option<String> {
    let value = properties.get(key)?;
    match <&str>::try_from(value) {
        Ok(text) => Some(text.to_owned()),
        Err(_) => <&zbus::zvariant::ObjectPath<'_>>::try_from(value)
            .ok()
            .map(|path| path.to_string()),
    }
}
