use std::sync::Arc;

use zbus::zvariant::OwnedObjectPath;
use zbus::{Connection, interface};

use crate::service::shared::Shared;

pub struct Session {
    shared: Arc<Shared>,
    path: OwnedObjectPath,
}

impl Session {
    pub fn new(shared: Arc<Shared>, path: OwnedObjectPath) -> Self {
        Self { shared, path }
    }
}

#[interface(name = "org.freedesktop.Secret.Session")]
impl Session {
    async fn close(&self, #[zbus(connection)] connection: &Connection) {
        self.shared.close_session(&self.path);
        let connection = connection.clone();
        let path = self.path.clone();
        tokio::spawn(async move {
            if let Err(error) = connection.object_server().remove::<Session, _>(&path).await {
                eprintln!("sparky-keyring: não deu para encerrar {path}: {error}");
            }
        });
    }
}
