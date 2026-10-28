use std::collections::HashMap;

use zbus::fdo::ManagedObjects;
use zbus::zvariant::OwnedValue;

use crate::domain::bluetooth::{Adapter, Bluetooth, Device};

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
                path: path.to_string(),
                name: text(properties, "Alias")
                    .or_else(|| text(properties, "Name"))
                    .unwrap_or_else(|| "Adaptador Bluetooth".to_owned()),
                powered,
                discovering: flag(properties, "Discovering"),
                devices: devices(objects, path.as_str()),
                connected,
            };
            Some((path.as_str(), adapter))
        })
        .collect();
    adapters.sort_by_key(|(path, _)| *path);
    (!adapters.is_empty())
        .then(|| Bluetooth::Ready(adapters.into_iter().map(|(_, adapter)| adapter).collect()))
}

fn devices(objects: &ManagedObjects, adapter: &str) -> Vec<Device> {
    let mut devices: Vec<_> = objects
        .iter()
        .filter_map(|(path, interfaces)| {
            let properties = interfaces.get(DEVICE)?;
            if text(properties, "Adapter").as_deref() != Some(adapter) {
                return None;
            }
            Some(Device {
                path: path.to_string(),
                name: text(properties, "Alias")
                    .or_else(|| text(properties, "Name"))
                    .unwrap_or_else(|| "Dispositivo Bluetooth".to_owned()),
                paired: flag(properties, "Paired"),
                connected: flag(properties, "Connected"),
            })
        })
        .collect();
    devices.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then(b.paired.cmp(&a.paired))
            .then(a.name.cmp(&b.name))
    });
    devices
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
