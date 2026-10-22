use std::collections::HashMap;
use std::sync::Arc;

use zbus::{Connection, message::Header};

use super::{
    Error, caller,
    shared::Shared,
    wire::{self, WireIdentity},
};
use crate::domain::{Outcome, Secret};

pub struct Agent(pub Arc<Shared>);

#[zbus::interface(name = "org.freedesktop.PolicyKit1.AuthenticationAgent")]
impl Agent {
    #[expect(
        clippy::too_many_arguments,
        reason = "Assinatura D-Bus oficial do polkit com conexão e remetente injetados"
    )]
    async fn begin_authentication(
        &self,
        action_id: &str,
        message: &str,
        _icon_name: &str,
        _details: HashMap<String, String>,
        cookie: &str,
        identities: Vec<WireIdentity>,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), Error> {
        caller::verify(connection, &header).await?;
        let request = wire::request(
            action_id,
            message,
            cookie,
            identities,
            crate::native::username,
        )
        .map_err(Error::Failed)?;
        let mut lease = self.0.claim(&request.cookie)?;
        let _guard = tokio::select! {
            guard = self.0.gate.lock() => guard,
            _ = lease.cancellation.changed() => return Err(Error::Cancelled("Autenticação cancelada".to_owned())),
        };
        if *lease.cancellation.borrow() {
            return Err(Error::Cancelled("Autenticação cancelada".to_owned()));
        }
        let outcome = self.0.runner.run(request, lease.cancellation.clone()).await;
        match outcome {
            Outcome::Authorized => Ok(()),
            Outcome::Cancelled => Err(Error::Cancelled("Autenticação cancelada".to_owned())),
            Outcome::Denied => Err(Error::Failed("Não foi possível autenticar".to_owned())),
        }
    }

    async fn cancel_authentication(
        &self,
        cookie: &str,
        #[zbus(connection)] connection: &Connection,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), Error> {
        caller::verify(connection, &header).await?;
        self.0.cancel(&Secret::new(cookie.to_owned()));
        Ok(())
    }
}
