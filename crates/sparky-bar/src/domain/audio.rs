#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Output,
    Input,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDevice {
    pub name: String,
    pub label: String,
    pub channels: u8,
    pub volume: u16,
    pub muted: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AudioState {
    pub outputs: Vec<AudioDevice>,
    pub inputs: Vec<AudioDevice>,
    pub default_output: Option<String>,
    pub default_input: Option<String>,
}

impl AudioState {
    pub fn devices(&self, direction: Direction) -> &[AudioDevice] {
        match direction {
            Direction::Output => &self.outputs,
            Direction::Input => &self.inputs,
        }
    }

    pub fn default_name(&self, direction: Direction) -> Option<&str> {
        match direction {
            Direction::Output => self.default_output.as_deref(),
            Direction::Input => self.default_input.as_deref(),
        }
    }

    pub fn selected(&self, direction: Direction) -> Option<&AudioDevice> {
        let name = self.default_name(direction)?;
        self.devices(direction)
            .iter()
            .find(|device| device.name == name)
    }

    pub fn icon(&self) -> &'static str {
        match self.selected(Direction::Output) {
            None => "audio-volume-muted-symbolic",
            Some(device) if device.muted || device.volume == 0 => "audio-volume-muted-symbolic",
            Some(device) if device.volume < 34 => "audio-volume-low-symbolic",
            Some(device) if device.volume < 67 => "audio-volume-medium-symbolic",
            _ => "audio-volume-high-symbolic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Audio {
    Unavailable(String),
    Ready(AudioState),
}

impl Default for Audio {
    fn default() -> Self {
        Self::Unavailable("Aguardando o serviço de áudio".to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioRequest {
    Volume {
        direction: Direction,
        name: String,
        channels: u8,
        percent: u16,
    },
    Mute {
        direction: Direction,
        name: String,
        muted: bool,
    },
    Default {
        direction: Direction,
        name: String,
    },
}
