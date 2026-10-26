use std::time::Duration;

use iced::futures::channel::mpsc::Sender;
use iced::futures::{
    SinkExt, StreamExt,
    stream::{self, BoxStream},
};
use zbus::fdo::{ManagedObjects, ObjectManagerProxy};
use zbus::proxy::{CacheProperties, MethodFlags};
use zbus::{Connection, MatchRule, MessageStream, message::Type};

use crate::adapters::bluez_objects;
use crate::domain::bluetooth::Bluetooth;
use crate::ports::bluetooth::BluetoothSource;

#[cfg(test)]
#[path = "bluetooth_events_tests.rs"]
mod tests;

pub struct Bluez;

impl BluetoothSource for Bluez {
    fn subscribe(&self) -> BoxStream<'static, Option<Bluetooth>> {
        Box::pin(iced::stream::channel(16, async |mut output| {
            loop {
                let result = monitor(&mut output).await;
                if result.is_ok() || output.is_closed() {
                    return;
                }
                if output.send(unavailable()).await.is_err() {
                    return;
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }))
    }
}

async fn monitor(output: &mut Sender<Option<Bluetooth>>) -> zbus::Result<()> {
    let connection = zbus::connection::Builder::system()?
        .method_timeout(Duration::from_secs(3))
        .build()
        .await?;
    let rule = MatchRule::builder()
        .msg_type(Type::Signal)
        .sender("org.bluez")?
        .build();
    let signals = MessageStream::for_match_rule(rule, &connection, Some(64)).await?;
    let owner = MatchRule::builder()
        .msg_type(Type::Signal)
        .sender("org.freedesktop.DBus")?
        .interface("org.freedesktop.DBus")?
        .member("NameOwnerChanged")?
        .arg(0, "org.bluez")?
        .build();
    let owners = MessageStream::for_match_rule(owner, &connection, Some(8)).await?;
    let mut signals = stream::select(signals, owners);
    let mut previous = snapshot(&connection).await;
    if output.send(previous.clone()).await.is_err() {
        return Ok(());
    }
    while let Some(message) = signals.next().await {
        let message = message?;
        if !relevant(&message) {
            continue;
        }
        let current = snapshot(&connection).await;
        if current == previous {
            continue;
        }
        previous = current.clone();
        if output.send(current).await.is_err() {
            return Ok(());
        }
    }
    Err(zbus::Error::Failure(
        "a conexão do Bluetooth foi encerrada".to_owned(),
    ))
}

async fn snapshot(connection: &Connection) -> Option<Bluetooth> {
    read_objects(connection)
        .await
        .unwrap_or_else(|_| unavailable())
}

async fn read_objects(connection: &Connection) -> zbus::Result<Option<Bluetooth>> {
    let proxy = ObjectManagerProxy::builder(connection)
        .destination("org.bluez")?
        .path("/")?
        .cache_properties(CacheProperties::No)
        .build()
        .await?;
    let objects: Option<ManagedObjects> = proxy
        .inner()
        .call_with_flags("GetManagedObjects", MethodFlags::NoAutoStart.into(), &())
        .await?;
    Ok(objects.as_ref().and_then(bluez_objects::bluetooth))
}

fn unavailable() -> Option<Bluetooth> {
    let found = std::fs::read_dir("/sys/class/bluetooth")
        .ok()?
        .any(|entry| entry.is_ok());
    found.then_some(Bluetooth::Unavailable)
}

fn relevant(message: &zbus::Message) -> bool {
    if message
        .header()
        .member()
        .is_none_or(|member| member.as_str() != "PropertiesChanged")
    {
        return true;
    }
    let Ok((interface, changed, invalidated)) = message.body().deserialize::<(
        String,
        std::collections::HashMap<String, zbus::zvariant::OwnedValue>,
        Vec<String>,
    )>() else {
        return false;
    };
    if interface != bluez_objects::ADAPTER && interface != bluez_objects::DEVICE {
        return false;
    }
    [
        "Powered",
        "Connected",
        "Alias",
        "Name",
        "Adapter",
        "Discovering",
        "Paired",
    ]
    .iter()
    .any(|key| changed.contains_key(*key) || invalidated.iter().any(|value| value == key))
}
