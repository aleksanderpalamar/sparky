use super::{
    agent::PATH,
    context::{PairContext, Target},
    proxy,
};
use std::{sync::Arc, time::Duration};
use zbus::{Connection, zvariant::ObjectPath};

#[derive(Clone, Copy)]
enum Procedure {
    Pair,
    Connect,
}

pub async fn pair(
    connection: &Connection,
    context: Arc<PairContext>,
    device: String,
    name: String,
) -> zbus::Result<()> {
    operate(connection, context, device, name, Procedure::Pair).await
}

pub async fn connect(
    connection: &Connection,
    context: Arc<PairContext>,
    device: String,
) -> zbus::Result<()> {
    let name = proxy(connection, &device, "org.bluez.Device1")
        .await?
        .get_property("Alias")
        .await?;
    operate(connection, context, device, name, Procedure::Connect).await
}

async fn operate(
    connection: &Connection,
    context: Arc<PairContext>,
    device: String,
    name: String,
    procedure: Procedure,
) -> zbus::Result<()> {
    let owner = zbus::fdo::DBusProxy::new(connection)
        .await?
        .get_name_owner("org.bluez".try_into()?)
        .await?
        .to_string();
    {
        let mut target = context
            .target
            .lock()
            .map_err(|_| zbus::Error::Failure("Estado de pareamento indisponível".to_owned()))?;
        if target.is_some() {
            return Err(zbus::Error::Failure(
                "Já existe um pareamento em andamento".to_owned(),
            ));
        }
        *target = Some(Target {
            owner,
            device: device.clone(),
            name,
            canceled: false,
        });
    }
    let result = match tokio::time::timeout(
        Duration::from_secs(110),
        run(connection, &context, &device, procedure),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(zbus::Error::Failure(
            "A operação Bluetooth excedeu o tempo limite".to_owned(),
        )),
    };
    if result.is_err() {
        let _ = tokio::time::timeout(Duration::from_secs(3), cancel(connection, &context)).await;
    }
    context.cancel();
    if let Ok(mut target) = context.target.lock() {
        *target = None;
    }
    let manager = proxy(connection, "/org/bluez", "org.bluez.AgentManager1").await?;
    let _ = tokio::time::timeout(
        Duration::from_secs(3),
        manager.call::<_, _, ()>("UnregisterAgent", &(ObjectPath::try_from(PATH)?,)),
    )
    .await;
    result
}

async fn run(
    connection: &Connection,
    context: &PairContext,
    device: &str,
    procedure: Procedure,
) -> zbus::Result<()> {
    let manager = proxy(connection, "/org/bluez", "org.bluez.AgentManager1").await?;
    manager
        .call::<_, _, ()>(
            "RegisterAgent",
            &(ObjectPath::try_from(PATH)?, "KeyboardDisplay"),
        )
        .await?;
    let device = proxy(connection, device, "org.bluez.Device1").await?;
    if matches!(procedure, Procedure::Pair) {
        device.call::<_, _, ()>("Pair", &()).await?;
    }
    if !context.active() {
        return Err(zbus::Error::Failure("Pareamento cancelado".to_owned()));
    }
    if matches!(procedure, Procedure::Pair) {
        device.set_property("Trusted", true).await?;
    }
    device.call("Connect", &()).await
}

pub async fn cancel(connection: &Connection, context: &PairContext) -> zbus::Result<()> {
    let device = context
        .target
        .lock()
        .ok()
        .and_then(|target| target.as_ref().map(|target| target.device.clone()));
    let Some(device) = device else {
        return Ok(());
    };
    proxy(connection, &device, "org.bluez.Device1")
        .await?
        .call("CancelPairing", &())
        .await
}
