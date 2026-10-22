use std::collections::HashMap;

use zbus::{
    Connection, Proxy,
    zvariant::{OwnedObjectPath, OwnedValue, Str},
};

use super::{AUTHORITY, AUTHORITY_PATH, PATH};

pub type Subject = (String, HashMap<String, OwnedValue>);

pub async fn session_id(connection: &Connection) -> zbus::Result<String> {
    let manager = Proxy::new(
        connection,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await?;
    let path: OwnedObjectPath = manager
        .call("GetSessionByPID", &(std::process::id(),))
        .await?;
    let session = Proxy::new(
        connection,
        "org.freedesktop.login1",
        path,
        "org.freedesktop.login1.Session",
    )
    .await?;
    session.get_property("Id").await
}

pub fn subject(id: &str) -> Subject {
    (
        "unix-session".to_owned(),
        [(
            "session-id".to_owned(),
            OwnedValue::from(Str::from(id.to_owned())),
        )]
        .into(),
    )
}

pub async fn authority(connection: &Connection) -> zbus::Result<Proxy<'_>> {
    Proxy::new(
        connection,
        AUTHORITY,
        AUTHORITY_PATH,
        "org.freedesktop.PolicyKit1.Authority",
    )
    .await
}

pub async fn register(authority: &Proxy<'_>, subject: &Subject) -> zbus::Result<()> {
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
        .unwrap_or_else(|| "en_US.UTF-8".to_owned());
    authority
        .call("RegisterAuthenticationAgent", &(subject, locale, PATH))
        .await
}

pub async fn unregister(authority: &Proxy<'_>, subject: &Subject) -> zbus::Result<()> {
    authority
        .call("UnregisterAuthenticationAgent", &(subject, PATH))
        .await
}
