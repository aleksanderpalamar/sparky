use super::{PRIVATE_PATH, PUBLIC_NAME, Shared, private::Controls};
use iced::futures::StreamExt;
use std::{sync::Arc, time::Duration};
use zbus::{Connection, MatchRule, MessageStream, object_server::SignalEmitter};

pub async fn pump(
    connection: &Connection,
    shared: Arc<Shared>,
    mut changed: impl FnMut() -> bool,
) -> zbus::Result<()> {
    let rule = MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .sender("org.freedesktop.DBus")?
        .interface("org.freedesktop.DBus")?
        .member("NameOwnerChanged")?
        .build();
    let mut owners = MessageStream::for_match_rule(rule, connection, Some(32)).await?;
    let mut updates = shared.subscribe();
    let mut last = shared
        .read(|center| center.snapshot_revision())
        .map_err(|error| zbus::Error::Failure(error.to_string()))?;
    if !changed() {
        return Ok(());
    }
    loop {
        let now = shared.now().elapsed_ms;
        let deadline = shared
            .read(|center| center.next_deadline())
            .map_err(|error| zbus::Error::Failure(error.to_string()))?;
        tokio::select! {
            result = updates.changed() => { if result.is_err() { return Ok(()); } }
            _ = expire_at(deadline,now) => {
                let events = shared.mutate(|center| center.expire(shared.now().elapsed_ms)).map_err(|error| zbus::Error::Failure(error.to_string()))?;
                super::emit_closed(connection,events).await?;
            }
            message = owners.next() => {
                let Some(message) = message else { return Err(zbus::Error::Failure("A conexão de notificações foi encerrada".to_owned())); };
                let message = message?;
                let (name,old,new): (String,String,String) = message.body().deserialize()?;
                if name == PUBLIC_NAME && new != connection.unique_name().map_or("", |name| name.as_str()) { return Err(zbus::Error::NameTaken); }
                if !name.starts_with(':') || !new.is_empty() { continue; }
                let events = shared.mutate(|center| center.owner_lost(&old)).map_err(|error| zbus::Error::Failure(error.to_string()))?;
                super::emit_closed(connection,events).await?;
            }
        }
        let revision = shared
            .read(|center| center.snapshot_revision())
            .map_err(|error| zbus::Error::Failure(error.to_string()))?;
        if revision == last {
            continue;
        }
        last = revision;
        Controls::changed(&SignalEmitter::new(connection, PRIVATE_PATH)?, revision).await?;
        if !changed() {
            return Ok(());
        }
    }
}

async fn expire_at(deadline: Option<u64>, now: u64) {
    match deadline {
        Some(deadline) => {
            tokio::time::sleep(Duration::from_millis(deadline.saturating_sub(now).max(1))).await
        }
        None => std::future::pending::<()>().await,
    }
}
