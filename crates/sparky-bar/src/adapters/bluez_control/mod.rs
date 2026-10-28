mod agent;
mod context;
mod discovery;
mod pairing;

use crate::{
    adapters::prompts::Broker, domain::bluetooth::BluetoothRequest,
    ports::bluetooth::BluetoothControl,
};
use context::PairContext;
use discovery::Discoveries;
use iced::futures::future::BoxFuture;
use std::{sync::Arc, time::Duration};
use zbus::{Connection, Proxy, proxy::CacheProperties};

pub struct BluezControl {
    connection: tokio::sync::Mutex<Option<Connection>>,
    discoveries: Arc<Discoveries>,
    pairing: Arc<PairContext>,
}

impl std::fmt::Debug for BluezControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BluezControl")
    }
}

impl BluezControl {
    pub fn new(prompts: Arc<Broker>) -> Self {
        Self {
            connection: Default::default(),
            discoveries: Arc::new(Discoveries::default()),
            pairing: Arc::new(PairContext::new(prompts)),
        }
    }

    async fn connection(&self) -> zbus::Result<Connection> {
        let mut stored = self.connection.lock().await;
        if let Some(connection) = stored.as_ref() {
            return Ok(connection.clone());
        }
        let connection = zbus::connection::Builder::system()?
            .method_timeout(Duration::from_secs(120))
            .build()
            .await?;
        connection
            .object_server()
            .at(agent::PATH, agent::Agent(self.pairing.clone()))
            .await?;
        *stored = Some(connection.clone());
        Ok(connection)
    }

    async fn execute(&self, request: BluetoothRequest) -> zbus::Result<()> {
        let connection = self.connection().await?;
        match request {
            BluetoothRequest::Power { adapter, enabled } => {
                self.discoveries.stop(&connection, &adapter, None).await?;
                proxy(&connection, &adapter, "org.bluez.Adapter1")
                    .await?
                    .set_property("Powered", enabled)
                    .await
                    .map_err(Into::into)
            }
            BluetoothRequest::Discover(adapter) => {
                self.discoveries.start(connection, adapter).await
            }
            BluetoothRequest::StopDiscovery(adapter) => {
                self.discoveries.stop(&connection, &adapter, None).await
            }
            BluetoothRequest::Connect(device) => {
                pairing::connect(&connection, self.pairing.clone(), device).await
            }
            BluetoothRequest::Disconnect(device) => {
                proxy(&connection, &device, "org.bluez.Device1")
                    .await?
                    .call("Disconnect", &())
                    .await
            }
            BluetoothRequest::Pair { device, name } => {
                pairing::pair(&connection, self.pairing.clone(), device, name).await
            }
            BluetoothRequest::Cancel => {
                self.pairing.cancel();
                self.discoveries.stop_all(&connection).await;
                pairing::cancel(&connection, &self.pairing).await
            }
        }
    }
}

impl BluetoothControl for BluezControl {
    fn send(&self, request: BluetoothRequest) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            self.execute(request)
                .await
                .map_err(|error| format!("Bluetooth: {error}"))
        })
    }
}

pub async fn proxy<'a>(
    connection: &'a Connection,
    path: &'a str,
    interface: &'a str,
) -> zbus::Result<Proxy<'a>> {
    if !path.starts_with("/org/bluez/") && path != "/org/bluez" {
        return Err(zbus::Error::Failure(
            "Caminho Bluetooth inválido".to_owned(),
        ));
    }
    zbus::proxy::Builder::new(connection)
        .destination("org.bluez")?
        .path(path)?
        .interface(interface)?
        .cache_properties(CacheProperties::No)
        .build()
        .await
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod pairing_tests;
