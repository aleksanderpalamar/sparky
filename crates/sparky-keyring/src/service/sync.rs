use std::sync::Arc;

use zbus::Connection;
use zbus::object_server::SignalEmitter;

use crate::domain::vault::Stored;
use crate::service::collection::Collection;
use crate::service::item::ItemObject;
use crate::service::paths;
use crate::service::shared::Shared;

pub async fn register(connection: &Connection, shared: &Arc<Shared>) -> zbus::Result<()> {
    let changes = shared.reconcile().map_err(|error| zbus::Error::Failure(error.to_string()))?;
    let server = connection.object_server();
    for id in changes.added {
        server.at(paths::item(id), ItemObject::new(id, Arc::clone(shared))).await?;
    }
    for id in changes.removed {
        server.remove::<ItemObject, _>(paths::item(id)).await?;
    }
    Ok(())
}

pub async fn refresh(connection: &Connection, shared: &Arc<Shared>) -> zbus::Result<()> {
    register(connection, shared).await?;
    let server = connection.object_server();
    for path in [paths::COLLECTION, paths::DEFAULT_ALIAS] {
        let collection = server.interface::<_, Collection>(path).await?;
        let emitter = collection.signal_emitter();
        let current = collection.get().await;
        current.locked_changed(emitter).await?;
        current.items_changed(emitter).await?;
        current.modified_changed(emitter).await?;
    }
    for id in shared.registered() {
        let item = server.interface::<_, ItemObject>(paths::item(id)).await?;
        let emitter = item.signal_emitter();
        let current = item.get().await;
        current.locked_changed(emitter).await?;
        current.label_changed(emitter).await?;
        current.attributes_changed(emitter).await?;
    }
    Ok(())
}

pub async fn stored(connection: &Connection, shared: &Arc<Shared>, stored: Stored) -> zbus::Result<()> {
    register(connection, shared).await?;
    let emitter = SignalEmitter::new(connection, paths::COLLECTION)?;
    let path = paths::item(stored.id());
    match stored {
        Stored::Created(_) => Collection::item_created(&emitter, path.as_ref()).await,
        Stored::Replaced(_) => Collection::item_changed(&emitter, path.as_ref()).await,
    }
}

pub async fn deleted(connection: &Connection, shared: &Arc<Shared>, id: u64) -> zbus::Result<()> {
    register(connection, shared).await?;
    let emitter = SignalEmitter::new(connection, paths::COLLECTION)?;
    Collection::item_deleted(&emitter, paths::item(id).as_ref()).await
}

pub async fn changed(connection: &Connection, id: u64) -> zbus::Result<()> {
    let emitter = SignalEmitter::new(connection, paths::COLLECTION)?;
    Collection::item_changed(&emitter, paths::item(id).as_ref()).await
}
