use super::{ROOT, proxy};
use crate::domain::network::{Security, WifiNetwork};
use std::collections::HashMap;
use zbus::{
    Connection,
    zvariant::{OwnedObjectPath, OwnedValue, Value},
};

pub type Settings = HashMap<String, HashMap<String, OwnedValue>>;

pub struct Profile {
    pub path: String,
    pub ssid: Vec<u8>,
    pub security: Security,
}

pub async fn read(connection: &Connection) -> zbus::Result<Vec<Profile>> {
    let path = format!("{ROOT}/Settings");
    let proxy = proxy(connection, &path, "org.freedesktop.NetworkManager.Settings").await?;
    let paths: Vec<OwnedObjectPath> = proxy.call("ListConnections", &()).await?;
    let mut profiles = Vec::new();
    for path in paths {
        let settings = settings(connection, path.as_str()).await?;
        let Some(wifi) = settings.get("802-11-wireless") else {
            continue;
        };
        let Some(ssid) = wifi
            .get("ssid")
            .and_then(|value| value.try_clone().ok())
            .and_then(|value| Vec::<u8>::try_from(value).ok())
        else {
            continue;
        };
        let security = security(&settings);
        profiles.push(Profile {
            path: path.to_string(),
            ssid,
            security,
        });
    }
    Ok(profiles)
}

pub fn security(settings: &Settings) -> Security {
    let key = settings
        .get("802-11-wireless-security")
        .and_then(|settings| settings.get("key-mgmt"))
        .and_then(|value| <&str>::try_from(value).ok());
    match key {
        None => Security::Open,
        Some("wpa-psk") => Security::Personal,
        Some("sae") => Security::Sae,
        Some("wpa-eap" | "wpa-eap-suite-b-192" | "ieee8021x") => Security::Enterprise,
        _ => Security::Unsupported,
    }
}

pub async fn settings(connection: &Connection, path: &str) -> zbus::Result<Settings> {
    proxy(
        connection,
        path,
        "org.freedesktop.NetworkManager.Settings.Connection",
    )
    .await?
    .call("GetSettings", &())
    .await
}

pub fn create(network: &WifiNetwork, password: &str, remember: bool) -> zbus::Result<Settings> {
    let mut settings = Settings::new();
    settings.insert(
        "connection".to_owned(),
        HashMap::from([
            ("id".to_owned(), owned(network.label())?),
            ("type".to_owned(), owned("802-11-wireless")?),
            ("autoconnect".to_owned(), remember.into()),
        ]),
    );
    settings.insert(
        "802-11-wireless".to_owned(),
        HashMap::from([
            ("ssid".to_owned(), owned(network.ssid.clone())?),
            ("mode".to_owned(), owned("infrastructure")?),
        ]),
    );
    if network.security == Security::Open {
        return Ok(settings);
    }
    let key = match network.security {
        Security::Sae => "sae",
        _ => "wpa-psk",
    };
    settings.insert(
        "802-11-wireless-security".to_owned(),
        HashMap::from([
            ("key-mgmt".to_owned(), owned(key)?),
            ("psk".to_owned(), owned(password)?),
            ("psk-flags".to_owned(), 0u32.into()),
        ]),
    );
    Ok(settings)
}

fn owned<'a>(value: impl Into<Value<'a>>) -> zbus::Result<OwnedValue> {
    Ok(value.into().try_into()?)
}
