use super::{Shared, wire};
use std::{collections::HashMap, sync::Arc};
use zbus::{fdo, message::Header, object_server::SignalEmitter, zvariant::OwnedValue};

pub struct Notifications(pub Arc<Shared>);

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl Notifications {
    fn get_capabilities(&self) -> Vec<&str> {
        vec!["body", "actions", "icon-static"]
    }

    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        ("Sparky", "Sparky", env!("CARGO_PKG_VERSION"), "1.3")
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Assinatura obrigatória do protocolo freedesktop Notifications"
    )]
    async fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
        #[zbus(header)] header: Header<'_>,
    ) -> fdo::Result<u32> {
        let sender = wire::sender(&header)?;
        let mut input = wire::incoming(
            crate::domain::Incoming {
                sender,
                app: app_name,
                replaces: replaces_id,
                icon: app_icon,
                title: summary,
                body,
                timeout: expire_timeout,
                ..Default::default()
            },
            actions,
            &hints,
        );
        let (picture, icon) = match self.0.images.clone().try_acquire_owned() {
            Ok(permit) => {
                let picture = super::pictures::prepare(&hints, &input.icon);
                let icon = input.icon.clone();
                tokio::task::spawn_blocking(move || {
                    let _permit = permit;
                    let icon = super::icons::resolve(&icon);
                    let picture = picture.resolve().or_else(|| super::pictures::file(&icon));
                    let icon = if icon.ends_with(".svg") {
                        icon
                    } else {
                        String::new()
                    };
                    (picture, icon)
                })
                .await
                .map_err(|_| fdo::Error::Failed("Não foi possível preparar a imagem".to_owned()))?
            }
            Err(_) => (None, String::new()),
        };
        input.picture = picture;
        input.icon = icon;
        self.0
            .mutate(|center| center.notify(input, self.0.now()))
            .map_err(wire::error)?
            .map_err(wire::error)
    }

    async fn close_notification(
        &self,
        id: u32,
        #[zbus(header)] header: Header<'_>,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        let sender = wire::sender(&header)?;
        let (id, reason) = self
            .0
            .mutate(|center| center.close_client(id, &sender))
            .map_err(wire::error)?
            .map_err(wire::error)?;
        Self::notification_closed(&emitter, id, reason)
            .await
            .map_err(Into::into)
    }

    #[zbus(signal)]
    pub async fn notification_closed(
        emitter: &SignalEmitter<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    pub async fn action_invoked(
        emitter: &SignalEmitter<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;
}
