use crate::domain::system::volumes::{self, Volume};
use crate::domain::system::{Change, Control, Request, Row, RowId};
use crate::domain::value::Step;
use crate::ports::commands::CommandRunner;

#[derive(Debug, Clone, Default)]
pub struct Media {
    volumes: Vec<Volume>,
    error: Option<String>,
}

impl Media {
    pub fn load(commands: &dyn CommandRunner) -> Self {
        let arguments = ["-J", "-o", "NAME,PATH,SIZE,TYPE,MOUNTPOINT,LABEL,RM,HOTPLUG,FSTYPE"];
        match commands.run("lsblk", &arguments) {
            Ok(output) => match volumes::parse(&output) {
                Ok(volumes) => Self { volumes, error: None },
                Err(_) => Self { error: Some("resposta inválida do lsblk".to_owned()), ..Self::default() },
            },
            Err(error) => Self { error: Some(error.to_string()), ..Self::default() },
        }
    }

    pub fn rows(&self) -> Vec<Row> {
        if let Some(error) = &self.error {
            return vec![Row { id: RowId::MediaStatus, label: "Volumes externos".to_owned(), value: error.clone(), control: Control::Reading }];
        }
        self.volumes.iter().enumerate().map(|(index, volume)| Row {
            id: RowId::Volume(index),
            label: format!("{} · {} · {}", volume.name(), volume.size, volume.filesystem),
            value: volume.mountpoint.as_ref().map(|point| format!("montado em {point}")).unwrap_or_else(|| "não montado".to_owned()),
            control: Control::Toggle(volume.mounted()),
        }).collect()
    }

    pub fn request(&self, id: RowId, change: Change) -> Option<Request> {
        let RowId::Volume(index) = id else {
            return None;
        };
        let volume = self.volumes.get(index)?;
        let mounted = match change {
            Change::Activate => !volume.mounted(),
            Change::Step(Step::More) => true,
            Change::Step(Step::Less) => false,
            Change::Text(_) => return None,
        };
        match (mounted, volume.mounted()) {
            (true, false) => Some(Request::MountVolume(volume.path.clone())),
            (false, true) => Some(Request::UnmountVolume(volume.path.clone())),
            _ => None,
        }
    }
}
