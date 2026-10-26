use super::{
    ROOT, SERVICE,
    agent::{Agent, PATH},
    profiles::{Settings, security},
    proxy,
};
use crate::{adapters::prompts::Broker, domain::panel::PanelKind, ports::prompts::Prompts};
use std::sync::Arc;
use zbus::Connection;

pub struct AgentGuard<'a> {
    connection: &'a Connection,
    prompts: Arc<Broker>,
}

impl<'a> AgentGuard<'a> {
    pub async fn register(
        connection: &'a Connection,
        prompts: Arc<Broker>,
        settings: &Settings,
        profile: Option<String>,
    ) -> zbus::Result<Self> {
        let owner = zbus::fdo::DBusProxy::new(connection)
            .await?
            .get_name_owner(SERVICE.try_into()?)
            .await?
            .to_string();
        let ssid = settings
            .get("802-11-wireless")
            .and_then(|settings| settings.get("ssid"))
            .and_then(|value| value.try_clone().ok())
            .and_then(|value| Vec::<u8>::try_from(value).ok())
            .unwrap_or_default();
        let agent = Agent {
            owner,
            profile,
            ssid,
            security: security(settings),
            prompts: prompts.clone(),
        };
        connection.object_server().at(PATH, agent).await?;
        let path = format!("{ROOT}/AgentManager");
        proxy(
            connection,
            &path,
            "org.freedesktop.NetworkManager.AgentManager",
        )
        .await?
        .call::<_, _, ()>("Register", &("org.sparky.bar",))
        .await?;
        Ok(Self {
            connection,
            prompts,
        })
    }

    pub async fn unregister(self) {
        self.prompts.cancel(PanelKind::Network);
        let path = format!("{ROOT}/AgentManager");
        let Ok(manager) = proxy(
            self.connection,
            &path,
            "org.freedesktop.NetworkManager.AgentManager",
        )
        .await
        else {
            return;
        };
        let _ = manager.call::<_, _, ()>("Unregister", &()).await;
    }
}
