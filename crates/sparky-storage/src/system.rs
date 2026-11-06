use std::process::Command;

use crate::{Operation, Volume, VolumeSource, parse};

pub const LIST_ARGUMENTS: &[&str] = &[
    "-J",
    "-o",
    "NAME,PATH,SIZE,TYPE,MOUNTPOINT,LABEL,RM,HOTPLUG,FSTYPE,MOUNTPOINTS,VENDOR,MODEL",
];

pub trait Commands: Send + Sync {
    fn output(&self, program: &str, arguments: &[&str]) -> Result<String, String>;
}

pub struct SystemCommands;

impl Commands for SystemCommands {
    fn output(&self, program: &str, arguments: &[&str]) -> Result<String, String> {
        let output = command(program, arguments)
            .output()
            .map_err(|error| format!("{program}: {error}"))?;
        if !output.status.success() {
            let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let detail = match message.is_empty() {
                true => output.status.to_string(),
                false => message,
            };
            return Err(failure(program, &detail));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

pub(crate) fn command(program: &str, arguments: &[&str]) -> Command {
    let mut process = match program {
        "udisksctl" => {
            let mut process = Command::new("setsid");
            process.args(["--wait", "--", program]);
            process
        }
        _ => Command::new(program),
    };
    process.args(arguments);
    process
}

pub(crate) fn failure(program: &str, detail: &str) -> String {
    if program != "udisksctl" {
        return format!("{program}: {detail}");
    }
    if detail.contains("NotAuthorizedDismissed") {
        return "Autenticação cancelada. O dispositivo não foi alterado.".to_owned();
    }
    if detail.contains(".NotAuthorized") || detail.contains("Not authorized") {
        return "Operação não autorizada. Verifique o agente gráfico polkit e tente novamente."
            .to_owned();
    }
    if detail.contains(".DeviceBusy") || detail.contains("Device is busy") {
        return "Dispositivo em uso. Feche os arquivos abertos e tente novamente.".to_owned();
    }
    format!("{program}: {detail}")
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
        match &operation {
            Operation::Eject(_) if !volume.can_eject() => {
                return Err(format!(
                    "ejeção não permitida para este dispositivo: {device}"
                ));
            }
            Operation::Mount(_) if volume.mounted() => return Ok(volumes),
            Operation::Unmount(_) if !volume.mounted() => return Ok(volumes),
            _ => {}
        }
        self.0.output("udisksctl", &[action, "-b", device])?;
        self.list()
    }
}
