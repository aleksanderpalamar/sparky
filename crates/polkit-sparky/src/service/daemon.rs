use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use zbus::Proxy;

use super::{AUTHORITY, PATH, agent::Agent, process::PromptProcess, registration, shared::Shared};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    crate::native::available()?;
    let shared = Shared::new(Arc::new(PromptProcess(std::env::current_exe()?)));
    let connection = zbus::connection::Builder::system()?
        .serve_at(PATH, Agent(shared.clone()))?
        .build()
        .await?;
    let id = registration::session_id(&connection).await?;
    let subject = registration::subject(&id);
    let authority = registration::authority(&connection).await?;
    let dbus = Proxy::new(
        &connection,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
    )
    .await?;
    let login = Proxy::new(
        &connection,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await?;
    let mut owners = dbus.receive_signal("NameOwnerChanged").await?;
    let mut sessions = login.receive_signal("SessionRemoved").await?;
    registration::register(&authority, &subject).await?;
    let mut owner: String = dbus.call("GetNameOwner", &(AUTHORITY,)).await?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut failure = None;
    let display = super::display::closed();
    tokio::pin!(display);
    loop {
        tokio::select! {
            _ = &mut display => break,
            _ = tokio::signal::ctrl_c() => break,
            _ = terminate.recv() => break,
            signal = owners.next() => {
                let Some(signal) = signal else { break };
                let Ok((name, _, next)) = signal.body().deserialize::<(String, String, String)>() else { continue };
                if name != AUTHORITY || next == owner {
                    continue;
                }
                owner = next;
                shared.cancel_all();
                if owner.is_empty() {
                    continue;
                }
                if let Err(error) = registration::register(&authority, &subject).await {
                    failure = Some(error);
                    break;
                }
            }
            signal = sessions.next() => {
                let Some(signal) = signal else { break };
                let Ok((removed, _)) = signal.body().deserialize::<(String, zbus::zvariant::OwnedObjectPath)>() else { continue };
                if removed == id {
                    break;
                }
            }
        }
    }
    shared.cancel_all();
    let _ = tokio::time::timeout(Duration::from_secs(4), shared.idle()).await;
    let _ = registration::unregister(&authority, &subject).await;
    match failure {
        Some(error) => Err(error.into()),
        None => Ok(()),
    }
}
