use iced::futures::future::BoxFuture;
use iced::futures::stream::BoxStream;

use crate::domain::bluetooth::Bluetooth;

pub trait BluetoothControl: std::fmt::Debug + Send + Sync {
    fn send(
        &self,
        request: crate::domain::bluetooth::BluetoothRequest,
    ) -> BoxFuture<'_, Result<(), String>>;
}

pub trait BluetoothSource {
    fn subscribe(&self) -> BoxStream<'static, Option<Bluetooth>>;
}
