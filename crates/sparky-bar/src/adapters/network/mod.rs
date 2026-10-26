mod agent;
mod agent_guard;
mod commands;
mod profiles;
mod snapshot;
mod wifi;

use std::sync::Arc;
use std::time::Duration;

use iced::futures::{SinkExt, StreamExt, future::BoxFuture, stream::BoxStream};
use zbus::proxy::CacheProperties;
use zbus::{Connection, MatchRule, MessageStream, Proxy};

use crate::adapters::prompts::Broker;
use crate::domain::network::{Network, NetworkRequest};
use crate::ports::network::NetworkControl;

pub const SERVICE: &str = "org.freedesktop.NetworkManager";
pub const ROOT: &str = "/org/freedesktop/NetworkManager";

pub struct NetworkManager {
    prompts: Arc<Broker>,
    operation: tokio::sync::Mutex<()>,
}

impl std::fmt::Debug for NetworkManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NetworkManager")
    }
}

impl NetworkManager {
    pub fn new(prompts: Arc<Broker>) -> Self {
        Self {
            prompts,
            operation: tokio::sync::Mutex::new(()),
        }
    }
}

pub async fn proxy<'a>(
    connection: &'a Connection,
    path: &'a str,
    interface: &'a str,
) -> zbus::Result<Proxy<'a>> {
    zbus::proxy::Builder::new(connection)
        .destination(SERVICE)?
        .path(path)?
        .interface(interface)?
        .cache_properties(CacheProperties::No)
        .build()
        .await
}

pub async fn connection() -> zbus::Result<Connection> {
    zbus::connection::Builder::system()?
        .method_timeout(Duration::from_secs(10))
        .build()
        .await
}

impl NetworkControl for NetworkManager {
    fn send(&self, request: NetworkRequest) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            let _operation = self
                .operation
                .try_lock()
                .map_err(|_| "Já existe uma operação de rede em andamento".to_owned())?;
            commands::send(request, self.prompts.clone())
                .await
                .map_err(|error| format!("Rede: {error}"))
        })
    }

    fn subscribe(&self) -> BoxStream<'static, Network> {
        Box::pin(iced::stream::channel(8, async |mut output| {
            loop {
                let result = monitor(&mut output).await;
                if result.is_ok() || output.is_closed() {
                    return;
                }
                let error = result
                    .err()
                    .map_or_else(String::new, |error| error.to_string());
                if output
                    .send(Network::Unavailable(format!(
                        "NetworkManager indisponível: {error}"
                    )))
                    .await
                    .is_err()
                {
                    return;
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }))
    }
}

async fn monitor(output: &mut iced::futures::channel::mpsc::Sender<Network>) -> zbus::Result<()> {
    let connection = connection().await?;
    let rule = MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .sender(SERVICE)?
        .build();
    let mut signals = MessageStream::for_match_rule(rule, &connection, Some(128)).await?;
    let mut previous = snapshot::read(&connection).await?;
    if output.send(Network::Ready(previous.clone())).await.is_err() {
        return Ok(());
    }
    loop {
        let event = tokio::time::timeout(Duration::from_secs(30), signals.next()).await;
        if matches!(event, Ok(None)) {
            return Err(zbus::Error::Failure("Conexão encerrada".to_owned()));
        }
        tokio::time::sleep(Duration::from_millis(120)).await;
        while let Some(Some(_)) = signals.next().now_or_never() {}
        let current = snapshot::read(&connection).await?;
        if current == previous {
            continue;
        }
        previous = current.clone();
        if output.send(Network::Ready(current)).await.is_err() {
            return Ok(());
        }
    }
}

use iced::futures::FutureExt;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod activation_tests;
