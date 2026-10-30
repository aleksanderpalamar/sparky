use crate::domain::audio::{Audio, AudioRequest, Direction};
use crate::domain::panel::PanelKind;

use super::panel_state::Panels;

#[derive(Debug, Clone)]
pub struct VolumeDraft {
    pub serial: u64,
    pub direction: Direction,
    pub name: String,
    pub channels: u8,
    pub percent: u16,
}

#[derive(Debug, Clone)]
pub struct AppVolumeDraft {
    pub serial: u64,
    pub index: u32,
    pub percent: u16,
}

impl Panels {
    pub fn finish_audio(&mut self, id: u64, kind: PanelKind) {
        if kind != PanelKind::Audio {
            return;
        }
        if self.volume.as_ref().is_some_and(|draft| draft.serial <= id) {
            self.volume = None;
        }
        if self
            .app_volume
            .as_ref()
            .is_some_and(|draft| draft.serial <= id)
        {
            self.app_volume = None;
        }
    }

    pub fn draft_volume(&mut self, direction: Direction, percent: u16) {
        let Audio::Ready(audio) = &self.audio else {
            return;
        };
        let Some(device) = audio.selected(direction) else {
            return;
        };
        self.sequence = self.sequence.wrapping_add(1);
        self.volume = Some(VolumeDraft {
            serial: self.sequence,
            direction,
            name: device.name.clone(),
            channels: device.channels,
            percent: percent.min(150),
        });
    }

    pub fn volume_request(&self) -> Option<AudioRequest> {
        let draft = self.volume.as_ref()?;
        Some(AudioRequest::Volume {
            direction: draft.direction,
            name: draft.name.clone(),
            channels: draft.channels,
            percent: draft.percent,
        })
    }

    pub fn draft_app_volume(&mut self, index: u32, percent: u16) {
        let Audio::Ready(audio) = &self.audio else {
            return;
        };
        if !audio
            .applications
            .iter()
            .any(|app| app.index == index && app.writable)
        {
            return;
        }
        self.sequence = self.sequence.wrapping_add(1);
        self.app_volume = Some(AppVolumeDraft {
            serial: self.sequence,
            index,
            percent: percent.min(150),
        });
    }

    pub fn app_volume_request(&self, index: u32) -> Option<AudioRequest> {
        let draft = self
            .app_volume
            .as_ref()
            .filter(|draft| draft.index == index)?;
        let Audio::Ready(audio) = &self.audio else {
            return None;
        };
        let app = audio
            .applications
            .iter()
            .find(|app| app.index == index && app.writable)?;
        Some(AudioRequest::AppVolume {
            index,
            channels: app.channels,
            percent: draft.percent,
        })
    }
}
