use super::profiles::{Settings, security};
use crate::{
    adapters::prompts::Broker,
    domain::{
        network::Security,
        panel::PanelKind,
        prompt::{Answer, PromptKind},
    },
    ports::prompts::Prompts,
};
use std::sync::Arc;
use zbus::{
    DBusError,
    message::Header,
    zvariant::{OwnedObjectPath, OwnedValue, Value},
};

pub const PATH: &str = "/org/freedesktop/NetworkManager/SecretAgent";

#[derive(Debug, DBusError)]
#[zbus(prefix = "org.freedesktop.NetworkManager.SecretAgent")]
pub enum Error {
    NoSecrets(String),
    UserCanceled(String),
    Failed(String),
}

pub struct Agent {
    pub owner: String,
    pub profile: Option<String>,
    pub ssid: Vec<u8>,
    pub security: Security,
    pub prompts: Arc<Broker>,
}

impl Agent {
    fn authorize(&self, header: &Header<'_>) -> Result<(), Error> {
        match header
            .sender()
            .is_some_and(|sender| sender.as_str() == self.owner)
        {
            true => Ok(()),
            false => Err(Error::Failed("Solicitante não autorizado".to_owned())),
        }
    }
}

#[zbus::interface(name = "org.freedesktop.NetworkManager.SecretAgent")]
impl Agent {
    async fn get_secrets(
        &self,
        settings: Settings,
        path: OwnedObjectPath,
        setting: String,
        _hints: Vec<String>,
        flags: u32,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<Settings, Error> {
        self.authorize(&header)?;
        let ssid = settings
            .get("802-11-wireless")
            .and_then(|settings| settings.get("ssid"))
            .and_then(|value| value.try_clone().ok())
            .and_then(|value| Vec::<u8>::try_from(value).ok());
        let matches = ssid.as_deref() == Some(self.ssid.as_slice())
            && self
                .profile
                .as_ref()
                .is_none_or(|profile| profile == path.as_str())
            && security(&settings) == self.security;
        if !matches
            || setting != "802-11-wireless-security"
            || flags & 1 == 0
            || !matches!(self.security, Security::Personal | Security::Sae)
        {
            return Err(Error::NoSecrets(
                "O painel não pode fornecer credenciais para esta conexão".to_owned(),
            ));
        }
        let answer = self
            .prompts
            .request(PromptKind::Wifi {
                ssid: String::from_utf8_lossy(&self.ssid).into_owned(),
            })
            .await;
        let Answer::Secret(secret) = answer else {
            return Err(Error::UserCanceled("Autenticação cancelada".to_owned()));
        };
        self.security
            .validate(secret.expose())
            .map_err(Error::NoSecrets)?;
        let psk: OwnedValue = Value::from(secret.expose())
            .try_into()
            .map_err(|error: zbus::zvariant::Error| Error::Failed(error.to_string()))?;
        let name: OwnedValue = Value::from(setting.as_str())
            .try_into()
            .map_err(|error: zbus::zvariant::Error| Error::Failed(error.to_string()))?;
        Ok([(
            setting,
            [("name".to_owned(), name), ("psk".to_owned(), psk)].into(),
        )]
        .into())
    }

    fn cancel_get_secrets(
        &self,
        _path: OwnedObjectPath,
        _setting: String,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), Error> {
        self.authorize(&header)?;
        self.prompts.cancel(PanelKind::Network);
        Ok(())
    }

    fn save_secrets(
        &self,
        _settings: Settings,
        _path: OwnedObjectPath,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), Error> {
        self.authorize(&header)?;
        Err(Error::NoSecrets(
            "O armazenamento de senhas é gerenciado pelo NetworkManager".to_owned(),
        ))
    }

    fn delete_secrets(
        &self,
        _settings: Settings,
        _path: OwnedObjectPath,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), Error> {
        self.authorize(&header)
    }
}
