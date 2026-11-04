use super::agent::Error;
use crate::{
    adapters::prompts::Broker,
    domain::{
        panel::PanelKind,
        prompt::{PromptId, PromptKind},
    },
    ports::prompts::Prompts,
};
use std::sync::{Arc, Mutex};
use zbus::message::Header;

pub struct Target {
    pub owner: String,
    pub device: String,
    pub name: String,
    pub canceled: bool,
}

pub struct PairContext {
    pub target: Mutex<Option<Target>>,
    pub prompts: Arc<Broker>,
    display: Mutex<Option<PromptId>>,
}

impl PairContext {
    pub fn active(&self) -> bool {
        self.target
            .lock()
            .ok()
            .is_some_and(|target| target.as_ref().is_some_and(|target| !target.canceled))
    }
    pub fn new(prompts: Arc<Broker>) -> Self {
        Self {
            target: Mutex::default(),
            prompts,
            display: Mutex::default(),
        }
    }

    pub fn authorize(&self, header: &Header<'_>, device: Option<&str>) -> Result<String, Error> {
        let target = self
            .target
            .lock()
            .map_err(|_| Error::Canceled(crate::i18n::text("pairing-unavailable")))?;
        let Some(target) = target.as_ref() else {
            return Err(Error::Rejected(crate::i18n::text("pairing-not-requested")));
        };
        if target.canceled {
            return Err(Error::Canceled(crate::i18n::text("pairing-canceled")));
        }
        let matches = header
            .sender()
            .is_some_and(|sender| sender.as_str() == target.owner)
            && device.is_none_or(|device| device == target.device);
        match matches {
            true => Ok(target.name.clone()),
            false => Err(Error::Rejected(crate::i18n::text(
                "pairing-invalid-requester",
            ))),
        }
    }

    pub fn cancel(&self) {
        if let Ok(mut target) = self.target.lock() {
            for target in target.iter_mut() {
                target.canceled = true;
            }
        }
        self.prompts.cancel(PanelKind::Bluetooth);
        let Ok(mut display) = self.display.lock() else {
            return;
        };
        if let Some(id) = display.take() {
            self.prompts.dismiss(id);
        }
    }

    pub fn display(&self, kind: PromptKind) -> Result<(), Error> {
        let mut display = self
            .display
            .lock()
            .map_err(|_| Error::Canceled(crate::i18n::text("panel-unavailable")))?;
        if let Some(id) = display.take() {
            self.prompts.dismiss(id);
        }
        *display = Some(self.prompts.display(kind));
        Ok(())
    }
}
