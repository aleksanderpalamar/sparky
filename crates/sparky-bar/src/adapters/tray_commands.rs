use iced::futures::future::BoxFuture;
use system_tray::client::{ActivateRequest, Client};

use crate::domain::tray::TrayRequest;
use crate::ports::tray::TrayCommands;

impl TrayCommands for Client {
    fn send(&self, request: TrayRequest) -> BoxFuture<'_, Result<(), String>> {
        let request = match request {
            TrayRequest::Activate { address } => ActivateRequest::Default { address, x: 0, y: 0 },
            TrayRequest::Select { address, menu_path, entry } => {
                ActivateRequest::MenuItem { address, menu_path, submenu_id: entry }
            }
        };
        Box::pin(async move { self.activate(request).await.map_err(|error| error.to_string()) })
    }
}
