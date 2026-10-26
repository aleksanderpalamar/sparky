use iced::futures::{future::BoxFuture, stream::BoxStream};

use crate::domain::network::{Network, NetworkRequest};

pub trait NetworkControl: std::fmt::Debug + Send + Sync {
    fn send(&self, request: NetworkRequest) -> BoxFuture<'_, Result<(), String>>;
    fn subscribe(&self) -> BoxStream<'static, Network>;
}
