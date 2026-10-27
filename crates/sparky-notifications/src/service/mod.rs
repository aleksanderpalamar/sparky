mod icons;
mod pictures;
mod private;
mod public;
mod pump;
pub mod readiness;
mod shared;
mod wire;

pub use pump::pump;
pub use shared::Shared;
use std::sync::Arc;
use zbus::{
    Connection,
    fdo::{RequestNameFlags, RequestNameReply},
    object_server::SignalEmitter,
};

pub const PUBLIC_NAME: &str = "org.freedesktop.Notifications";
pub const PUBLIC_PATH: &str = "/org/freedesktop/Notifications";
pub const PRIVATE_NAME: &str = "org.sparky.Notifications";
pub const PRIVATE_PATH: &str = "/org/sparky/Notifications";

pub async fn serve(connection: &Connection, shared: Arc<Shared>) -> zbus::Result<()> {
    connection
        .object_server()
        .at(PUBLIC_PATH, public::Notifications(shared.clone()))
        .await?;
    connection
        .object_server()
        .at(PRIVATE_PATH, private::Controls(shared))
        .await?;
    claim(connection, PUBLIC_NAME).await?;
    claim(connection, PRIVATE_NAME).await
}

async fn claim(connection: &Connection, name: &str) -> zbus::Result<()> {
    match connection
        .request_name_with_flags(name, RequestNameFlags::DoNotQueue.into())
        .await?
    {
        RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner => Ok(()),
        _ => Err(zbus::Error::NameTaken),
    }
}

pub async fn emit_closed(
    connection: &Connection,
    events: impl IntoIterator<Item = (u32, u32)>,
) -> zbus::Result<()> {
    let emitter = SignalEmitter::new(connection, PUBLIC_PATH)?;
    for (id, reason) in events {
        public::Notifications::notification_closed(&emitter, id, reason).await?;
    }
    Ok(())
}

#[cfg(test)]
mod test_bus;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod pump_tests;
