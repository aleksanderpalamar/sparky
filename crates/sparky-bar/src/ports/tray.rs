use iced::futures::future::BoxFuture;

use crate::domain::tray::TrayRequest;

pub trait TrayCommands: std::fmt::Debug + Send + Sync {
    fn send(&self, request: TrayRequest) -> BoxFuture<'_, Result<(), String>>;
}
