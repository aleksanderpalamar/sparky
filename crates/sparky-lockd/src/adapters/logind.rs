use std::os::fd::OwnedFd;

use futures_util::future::ready;
use futures_util::stream::{StreamExt, select};
use tokio::sync::mpsc::Sender;
use zbus::proxy::{Builder, CacheProperties};
use zbus::zvariant::{self, ObjectPath, OwnedObjectPath};
use zbus::{Connection, Proxy};

use crate::app::Event;
use crate::domain::guard::{Input, Screen};
use crate::ports::Logind;

pub const WHY: &str = "Bloquear a tela antes de suspender";
const SERVICE: &str = "org.freedesktop.login1";
const PATH: &str = "/org/freedesktop/login1";
const MANAGER: &str = "org.freedesktop.login1.Manager";
const SESSION: &str = "org.freedesktop.login1.Session";

pub struct SystemLogind {
    manager: Proxy<'static>,
    session: Proxy<'static>,
}

impl SystemLogind {
    pub async fn connect(connection: &Connection) -> zbus::Result<Self> {
        let manager = proxy(connection, ObjectPath::from_static_str(PATH)?, MANAGER).await?;
        let path: OwnedObjectPath = manager
            .call("GetSessionByPID", &(std::process::id(),))
            .await?;
        let session = proxy(connection, path.into_inner(), SESSION).await?;
        Ok(Self { manager, session })
    }

    pub async fn watch(&self, sender: Sender<Event>) -> zbus::Result<()> {
        let sleep = self
            .manager
            .receive_signal("PrepareForSleep")
            .await?
            .filter_map(|message| ready(message.body().deserialize::<bool>().ok().map(sleeping)));
        let lock = self
            .session
            .receive_signal("Lock")
            .await?
            .map(|_| Input::LockAsked);
        let unlock = self
            .session
            .receive_signal("Unlock")
            .await?
            .map(|_| Input::UnlockAsked);
        let mut inputs = select(sleep, select(lock, unlock));
        tokio::spawn(async move {
            while let Some(input) = inputs.next().await {
                if sender.send(Event::Seen(input)).await.is_err() {
                    return;
                }
            }
        });
        Ok(())
    }
}

impl Logind for SystemLogind {
    type Inhibitor = OwnedFd;

    async fn inhibit(&self) -> Result<OwnedFd, String> {
        let inhibitor: zvariant::OwnedFd = self
            .manager
            .call("Inhibit", &("sleep", "Sparky", WHY, "delay"))
            .await
            .map_err(|error| format!("logind não segurou a suspensão: {error}"))?;
        Ok(inhibitor.into())
    }

    async fn hint(&self, screen: Screen) -> Result<(), String> {
        let locked = matches!(screen, Screen::Locked);
        self.session
            .call::<_, _, ()>("SetLockedHint", &(locked,))
            .await
            .map_err(|error| format!("logind não guardou a dica de bloqueio: {error}"))
    }
}

fn sleeping(starting: bool) -> Input {
    match starting {
        true => Input::Sleeping,
        false => Input::Woke,
    }
}

async fn proxy(
    connection: &Connection,
    path: ObjectPath<'static>,
    interface: &'static str,
) -> zbus::Result<Proxy<'static>> {
    Builder::new(connection)
        .destination(SERVICE)?
        .path(path)?
        .interface(interface)?
        .cache_properties(CacheProperties::No)
        .build()
        .await
}
