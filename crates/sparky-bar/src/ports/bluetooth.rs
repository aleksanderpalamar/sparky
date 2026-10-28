use iced::futures::stream::BoxStream;

use crate::domain::bluetooth::Bluetooth;

pub trait BluetoothSource {
    fn subscribe(&self) -> BoxStream<'static, Option<Bluetooth>>;
}
