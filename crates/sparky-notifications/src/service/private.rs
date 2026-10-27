use super::{Shared, public::Notifications, wire};
use std::sync::Arc;
use zbus::{fdo, message::Header, object_server::SignalEmitter};

pub struct Controls(pub Arc<Shared>);

#[zbus::interface(name = "org.sparky.Notifications")]
impl Controls {
    fn snapshot(&self) -> fdo::Result<String> {
        serde_json::to_string(&self.0.snapshot().map_err(wire::error)?)
            .map_err(|error| fdo::Error::Failed(error.to_string()))
    }

    fn set_do_not_disturb(&self, enabled: bool) -> fdo::Result<()> {
        self.0
            .mutate(|center| center.set_dnd(enabled))
            .map_err(wire::error)
    }

    fn center_open(&self, open: bool, #[zbus(header)] header: Header<'_>) -> fdo::Result<()> {
        let sender = wire::sender(&header)?;
        self.0
            .mutate(|center| center.center_open(&sender, open))
            .map_err(wire::error)
    }

    async fn dismiss(
        &self,
        id: u32,
        #[zbus(connection)] connection: &zbus::Connection,
    ) -> fdo::Result<()> {
        let closed = self
            .0
            .mutate(|center| center.dismiss(id, 2))
            .map_err(wire::error)?;
        super::emit_closed(connection, closed)
            .await
            .map_err(Into::into)
    }

    async fn clear(&self, #[zbus(connection)] connection: &zbus::Connection) -> fdo::Result<()> {
        let closed = self
            .0
            .mutate(|center| center.clear())
            .map_err(wire::error)?;
        super::emit_closed(connection, closed)
            .await
            .map_err(Into::into)
    }

    async fn invoke(
        &self,
        id: u32,
        key: &str,
        #[zbus(connection)] connection: &zbus::Connection,
    ) -> fdo::Result<()> {
        let closed = self
            .0
            .mutate(|center| center.invoke(id, key))
            .map_err(wire::error)?
            .map_err(wire::error)?;
        let emitter = SignalEmitter::new(connection, super::PUBLIC_PATH)?;
        Notifications::action_invoked(&emitter, id, key).await?;
        super::emit_closed(connection, closed)
            .await
            .map_err(Into::into)
    }

    #[zbus(signal)]
    pub async fn changed(emitter: &SignalEmitter<'_>, revision: u64) -> zbus::Result<()>;
}
