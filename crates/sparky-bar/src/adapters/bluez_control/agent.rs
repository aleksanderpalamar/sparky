use super::context::PairContext;
use crate::domain::prompt::{Answer, PromptKind};
use std::sync::Arc;
use zbus::{DBusError, message::Header, zvariant::OwnedObjectPath};

pub const PATH: &str = "/org/sparky/BluetoothAgent";

#[derive(Debug, DBusError)]
#[zbus(prefix = "org.bluez.Error")]
pub enum Error {
    Rejected(String),
    Canceled(String),
}

pub struct Agent(pub Arc<PairContext>);

#[zbus::interface(name = "org.bluez.Agent1")]
impl Agent {
    fn release(&self, #[zbus(header)] header: Header<'_>) -> Result<(), Error> {
        self.0.authorize(&header, None)?;
        self.0.cancel();
        Ok(())
    }

    async fn request_pin_code(
        &self,
        device: OwnedObjectPath,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<String, Error> {
        let name = self.0.authorize(&header, Some(device.as_str()))?;
        let answer = self
            .0
            .prompts
            .request(PromptKind::Pin {
                device: name,
                numeric: false,
            })
            .await;
        match answer {
            Answer::Secret(secret) if (1..=16).contains(&secret.expose().len()) => {
                Ok(secret.expose().to_owned())
            }
            _ => Err(Error::Rejected("PIN inválido ou cancelado".to_owned())),
        }
    }

    async fn request_passkey(
        &self,
        device: OwnedObjectPath,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<u32, Error> {
        let name = self.0.authorize(&header, Some(device.as_str()))?;
        let answer = self
            .0
            .prompts
            .request(PromptKind::Pin {
                device: name,
                numeric: true,
            })
            .await;
        let Answer::Secret(secret) = answer else {
            return Err(Error::Rejected("Código cancelado".to_owned()));
        };
        parse_passkey(secret.expose())
            .ok_or_else(|| Error::Rejected("Digite até seis algarismos".to_owned()))
    }

    fn display_pin_code(
        &self,
        device: OwnedObjectPath,
        code: String,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), Error> {
        let name = self.0.authorize(&header, Some(device.as_str()))?;
        self.0.display(PromptKind::Display {
            device: name,
            code,
            entered: 0,
        })
    }

    fn display_passkey(
        &self,
        device: OwnedObjectPath,
        code: u32,
        entered: u16,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), Error> {
        let name = self.0.authorize(&header, Some(device.as_str()))?;
        self.0.display(PromptKind::Display {
            device: name,
            code: format!("{code:06}"),
            entered,
        })
    }

    async fn request_confirmation(
        &self,
        device: OwnedObjectPath,
        code: u32,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), Error> {
        let name = self.0.authorize(&header, Some(device.as_str()))?;
        accepted(
            self.0
                .prompts
                .request(PromptKind::Confirm {
                    device: name,
                    code: Some(code),
                })
                .await,
        )
    }

    async fn request_authorization(
        &self,
        device: OwnedObjectPath,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), Error> {
        let name = self.0.authorize(&header, Some(device.as_str()))?;
        accepted(
            self.0
                .prompts
                .request(PromptKind::Confirm {
                    device: name,
                    code: None,
                })
                .await,
        )
    }

    async fn authorize_service(
        &self,
        device: OwnedObjectPath,
        uuid: String,
        #[zbus(header)] header: Header<'_>,
    ) -> Result<(), Error> {
        let name = self.0.authorize(&header, Some(device.as_str()))?;
        accepted(
            self.0
                .prompts
                .request(PromptKind::Confirm {
                    device: format!("{name}\nServiço: {uuid}"),
                    code: None,
                })
                .await,
        )
    }

    fn cancel(&self, #[zbus(header)] header: Header<'_>) -> Result<(), Error> {
        self.0.authorize(&header, None)?;
        self.0.cancel();
        Ok(())
    }
}

fn accepted(answer: Answer) -> Result<(), Error> {
    match answer {
        Answer::Accept => Ok(()),
        _ => Err(Error::Rejected("Pareamento recusado".to_owned())),
    }
}

pub fn parse_passkey(value: &str) -> Option<u32> {
    if value.is_empty() || value.len() > 6 || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}
