use std::collections::HashMap;
use std::sync::Arc;

use iced::window;

use crate::domain::{
    audio::Audio,
    bluetooth::Bluetooth,
    network::{Network, NetworkRequest, WifiNetwork},
    panel::PanelKind,
    prompt::{Prompt, PromptId},
    secret::Secret,
};
use crate::ports::{audio::AudioControl, network::NetworkControl, prompts::Prompts};

#[derive(Debug, Clone)]
pub struct AudioClient(pub Arc<dyn AudioControl>);
#[derive(Debug, Clone)]
pub struct NetworkClient(pub Arc<dyn NetworkControl>);
#[derive(Debug, Clone)]
pub struct PromptClient(pub Arc<dyn Prompts>);
#[derive(Debug, Clone)]
pub struct BluetoothClient(pub Arc<dyn crate::ports::bluetooth::BluetoothControl>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenPanel {
    pub window: window::Id,
    pub kind: PanelKind,
}

#[derive(Debug, Clone)]
pub struct WifiPassword {
    pub network: WifiNetwork,
    pub password: Secret,
    pub remember: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Pending {
    pub id: u64,
    pub origin: Option<window::Id>,
}

#[derive(Debug, Default)]
pub struct Panels {
    pub input_focus: InputFocus,
    pub open: Option<OpenPanel>,
    pub audio: Audio,
    pub network: Network,
    pub bluetooth: Option<Bluetooth>,
    pub audio_client: Option<AudioClient>,
    pub network_client: Option<NetworkClient>,
    pub bluetooth_client: Option<BluetoothClient>,
    pub prompts: Option<PromptClient>,
    pub prompt: Option<Prompt>,
    pub prompt_input: Secret,
    pub wifi: Option<WifiPassword>,
    pub volume: Option<super::panel_audio::VolumeDraft>,
    pub app_volume: Option<super::panel_audio::AppVolumeDraft>,
    pub metrics: Option<crate::domain::telemetry::Metrics>,
    pub sampling: bool,
    pub pending: HashMap<PanelKind, Pending>,
    pub errors: HashMap<PanelKind, String>,
    pub(super) sequence: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputFocus {
    #[default]
    Interactive,
    Yielded,
    Restoring,
}

impl Panels {
    pub fn begin(&mut self, kind: PanelKind) -> u64 {
        self.sequence = self.sequence.wrapping_add(1);
        let origin = self
            .open
            .filter(|open| open.kind == kind)
            .map(|open| open.window);
        self.pending.insert(
            kind,
            Pending {
                id: self.sequence,
                origin,
            },
        );
        self.errors.remove(&kind);
        self.sequence
    }

    pub fn finish(&mut self, id: u64, kind: PanelKind, result: Result<(), String>) {
        let Some(pending) = self
            .pending
            .get(&kind)
            .copied()
            .filter(|pending| pending.id == id)
        else {
            return;
        };
        self.pending.remove(&kind);
        if pending.origin.is_some() && self.open.map(|open| open.window) != pending.origin {
            return;
        }
        match result {
            Ok(()) => {
                self.errors.remove(&kind);
            }
            Err(error) => {
                self.errors.insert(kind, error);
            }
        }
        self.finish_audio(id, kind);
    }

    pub fn clear_input(&mut self) {
        self.wifi = None;
        self.prompt = None;
        self.prompt_input = Secret::default();
        self.volume = None;
        self.app_volume = None;
    }

    pub fn dismiss(&mut self, id: PromptId) {
        if self.prompt.as_ref().is_none_or(|prompt| prompt.id != id) {
            return;
        }
        self.prompt = None;
        self.prompt_input = Secret::default();
    }

    pub fn wifi_request(&self) -> Option<NetworkRequest> {
        let wifi = self.wifi.as_ref()?;
        Some(NetworkRequest::Join {
            network: wifi.network.clone(),
            password: wifi.password.clone(),
            remember: wifi.remember,
        })
    }
}
