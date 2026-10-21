use std::sync::Arc;

use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedObjectPath, Value};
use zbus::{Connection, interface};

use crate::keyring::unlocking::Outcome;
use crate::service::shared::Shared;
use crate::service::sync;

pub struct Prompt {
    shared: Arc<Shared>,
    path: OwnedObjectPath,
    objects: Vec<OwnedObjectPath>,
}

impl Prompt {
    pub fn new(shared: Arc<Shared>, path: OwnedObjectPath, objects: Vec<OwnedObjectPath>) -> Self {
        Self { shared, path, objects }
    }
}

#[interface(name = "org.freedesktop.Secret.Prompt")]
impl Prompt {
    async fn prompt(&self, _window_id: &str, #[zbus(connection)] connection: &Connection) {
        let shared = Arc::clone(&self.shared);
        let connection = connection.clone();
        let path = self.path.clone();
        let objects = self.objects.clone();
        tokio::spawn(async move {
            let asking = Arc::clone(&shared);
            let outcome = tokio::task::spawn_blocking(move || asking.unlock()).await.unwrap_or(Outcome::Dismissed);
            report(finish(&connection, &shared, &path, outcome, objects).await, &path);
        });
    }

    async fn dismiss(&self, #[zbus(connection)] connection: &Connection) {
        let shared = Arc::clone(&self.shared);
        let connection = connection.clone();
        let path = self.path.clone();
        tokio::spawn(async move {
            report(finish(&connection, &shared, &path, Outcome::Dismissed, Vec::new()).await, &path);
        });
    }

    #[zbus(signal)]
    pub async fn completed(emitter: &SignalEmitter<'_>, dismissed: bool, result: Value<'_>) -> zbus::Result<()>;
}

async fn finish(
    connection: &Connection,
    shared: &Arc<Shared>,
    path: &OwnedObjectPath,
    outcome: Outcome,
    objects: Vec<OwnedObjectPath>,
) -> zbus::Result<()> {
    if outcome == Outcome::Unlocked {
        sync::refresh(connection, shared).await?;
    }
    let (dismissed, result) = match outcome {
        Outcome::Unlocked => (false, objects),
        Outcome::Dismissed => (true, Vec::new()),
    };
    let emitter = SignalEmitter::new(connection, path.clone())?;
    Prompt::completed(&emitter, dismissed, Value::from(result)).await?;
    connection.object_server().remove::<Prompt, _>(path).await?;
    Ok(())
}

fn report(result: zbus::Result<()>, path: &OwnedObjectPath) {
    if let Err(error) = result {
        eprintln!("sparky-keyring: o pedido {path} falhou: {error}");
    }
}
