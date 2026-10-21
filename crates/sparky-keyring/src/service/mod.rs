pub mod collection;
pub mod errors;
pub mod item;
pub mod paths;
pub mod portal;
pub mod prompt;
pub mod secrets;
pub mod session;
pub mod shared;
pub mod sync;
pub mod wire;

#[cfg(test)]
mod service_tests;
#[cfg(test)]
mod service_support;

use std::sync::Arc;

use zbus::Connection;

use crate::service::collection::Collection;
use crate::service::portal::Portal;
use crate::service::secrets::Service;
use crate::service::shared::Shared;

pub async fn serve(connection: &Connection, shared: Arc<Shared>) -> zbus::Result<()> {
    let server = connection.object_server();
    server.at(paths::SERVICE, Service::new(Arc::clone(&shared))).await?;
    server.at(paths::COLLECTION, Collection::new(Arc::clone(&shared))).await?;
    server.at(paths::DEFAULT_ALIAS, Collection::new(Arc::clone(&shared))).await?;
    server.at(portal::PATH, Portal::new(Arc::clone(&shared))).await?;
    sync::register(connection, &shared).await
}
