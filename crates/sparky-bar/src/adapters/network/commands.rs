use super::{ROOT, SERVICE, agent_guard::AgentGuard, connection, profiles::Settings, proxy};
use crate::{
    adapters::prompts::Broker,
    domain::network::{NetworkRequest, WifiNetwork},
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use zbus::{
    Connection,
    zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value},
};

pub async fn send(request: NetworkRequest, prompts: Arc<Broker>) -> zbus::Result<()> {
    let connection = connection().await?;
    send_on(&connection, request, prompts).await
}

pub async fn send_on(
    connection: &Connection,
    request: NetworkRequest,
    prompts: Arc<Broker>,
) -> zbus::Result<()> {
    match request {
        NetworkRequest::Wireless(enabled) => proxy(connection, ROOT, SERVICE)
            .await?
            .set_property("WirelessEnabled", enabled)
            .await
            .map_err(Into::into),
        NetworkRequest::Scan(device) => {
            proxy(
                connection,
                &device,
                "org.freedesktop.NetworkManager.Device.Wireless",
            )
            .await?
            .call("RequestScan", &(HashMap::<String, OwnedValue>::new(),))
            .await
        }
        NetworkRequest::Disconnect(device) => {
            proxy(connection, &device, "org.freedesktop.NetworkManager.Device")
                .await?
                .call("Disconnect", &())
                .await
        }
        NetworkRequest::Activate {
            profile,
            device,
            access_point,
        } => {
            if profile == "/" {
                return activate(connection, &profile, &device, &access_point).await;
            }
            let settings = super::profiles::settings(connection, &profile).await?;
            let guard =
                AgentGuard::register(connection, prompts, &settings, Some(profile.clone())).await?;
            let result = activate(connection, &profile, &device, &access_point).await;
            guard.unregister().await;
            result
        }
        NetworkRequest::Join {
            network,
            password,
            remember,
        } => {
            network
                .security
                .validate(password.expose())
                .map_err(zbus::Error::Failure)?;
            let settings = super::profiles::create(&network, password.expose(), remember)?;
            let guard = AgentGuard::register(connection, prompts, &settings, None).await?;
            let result = join(connection, settings, &network, remember).await;
            guard.unregister().await;
            result
        }
    }
}

async fn activate(
    connection: &Connection,
    profile: &str,
    device: &str,
    ap: &str,
) -> zbus::Result<()> {
    let active: OwnedObjectPath = proxy(connection, ROOT, SERVICE)
        .await?
        .call(
            "ActivateConnection",
            &(
                ObjectPath::try_from(profile)?,
                ObjectPath::try_from(device)?,
                ObjectPath::try_from(ap)?,
            ),
        )
        .await?;
    wait(connection, active.as_str()).await
}

async fn join(
    connection: &Connection,
    settings: Settings,
    network: &WifiNetwork,
    remember: bool,
) -> zbus::Result<()> {
    let persistence = if remember { "disk" } else { "volatile" };
    let options = HashMap::from([("persist", Value::from(persistence))]);
    let (_, active, _): (
        OwnedObjectPath,
        OwnedObjectPath,
        HashMap<String, OwnedValue>,
    ) = proxy(connection, ROOT, SERVICE)
        .await?
        .call(
            "AddAndActivateConnection2",
            &(
                settings,
                ObjectPath::try_from(network.device.as_str())?,
                ObjectPath::try_from(network.path.as_str())?,
                options,
            ),
        )
        .await?;
    wait(connection, active.as_str()).await
}

async fn wait(connection: &Connection, active: &str) -> zbus::Result<()> {
    let proxy = proxy(
        connection,
        active,
        "org.freedesktop.NetworkManager.Connection.Active",
    )
    .await?;
    let result = tokio::time::timeout(Duration::from_secs(110), async {
        loop {
            let state: u32 = proxy.get_property("State").await?;
            match state {
                2 => return Ok(()),
                3 | 4 => {
                    return Err(zbus::Error::Failure(format!(
                        "A conexão não foi concluída (estado {state})"
                    )));
                }
                _ => tokio::time::sleep(Duration::from_millis(500)).await,
            }
        }
    })
    .await;
    match result {
        Ok(result) => result,
        Err(_) => Err(zbus::Error::Failure(
            "A conexão excedeu o tempo limite; verifique a senha e o sinal".to_owned(),
        )),
    }
}
