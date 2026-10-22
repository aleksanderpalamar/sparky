use std::process::Command;

use crate::{Operation, Volume, VolumeSource, parse};

pub const LIST_ARGUMENTS: &[&str] = &[
    "-J",
    "-o",
    "NAME,PATH,SIZE,TYPE,MOUNTPOINT,LABEL,RM,HOTPLUG,FSTYPE,MOUNTPOINTS",
];

pub trait Commands: Send + Sync {
    fn output(&self, program: &str, arguments: &[&str]) -> Result<String, String>;
}

pub struct SystemCommands;

impl Commands for SystemCommands {
    fn output(&self, program: &str, arguments: &[&str]) -> Result<String, String> {
        let output = Command::new(program)
            .args(arguments)
            .output()
            .map_err(|error| format!("{program}: {error}"))?;
        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let detail = match message.is_empty() {
                true => output.status.to_string(),
                false => message,
            };
            return Err(format!("{program}: {detail}"));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

pub struct SystemVolumes<C = SystemCommands>(pub C);

impl<C: Commands> SystemVolumes<C> {
    fn list(&self) -> Result<Vec<Volume>, String> {
        let output = self.0.output("lsblk", LIST_ARGUMENTS)?;
        parse(&output).map_err(|error| format!("resposta inválida do lsblk: {error}"))
    }
}

impl<C: Commands> VolumeSource for SystemVolumes<C> {
    fn run(&self, operation: Operation) -> Result<Vec<Volume>, String> {
        let volumes = self.list()?;
        let Some((action, device)) = operation.command() else {
            return Ok(volumes);
        };
        let volume = volumes
            .iter()
            .find(|volume| volume.path == device)
            .ok_or_else(|| format!("dispositivo indisponível: {device}"))?;
        let desired = matches!(operation, Operation::Mount(_));
        if volume.mounted() == desired {
            return Ok(volumes);
        }
        self.0.output("udisksctl", &[action, "-b", device])?;
        self.list()
    }
}
