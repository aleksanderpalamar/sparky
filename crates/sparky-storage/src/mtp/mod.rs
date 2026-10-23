use crate::{Operation, Volume, VolumeSource};

pub mod gio_mounts;
mod identity;
pub mod udev_phones;

pub trait Phones: Send + Sync {
    fn connected(&self) -> Result<Vec<Volume>, String>;
}

pub trait Mounts: Send + Sync {
    fn mountpoint(&self, uri: &str) -> Result<Option<String>, String>;
    fn mount(&self, uri: &str) -> Result<(), String>;
    fn unmount(&self, uri: &str) -> Result<(), String>;
}

pub struct MtpVolumes<D, M> {
    pub devices: D,
    pub mounts: M,
}

impl<D: Phones, M: Mounts> VolumeSource for MtpVolumes<D, M> {
    fn run(&self, operation: Operation) -> Result<Vec<Volume>, String> {
        let devices = self.devices.connected()?;
        let Some((_, uri)) = operation.command() else {
            return self.with_mountpoints(devices);
        };
        if !devices.iter().any(|device| device.path == uri) {
            return Err("O telefone MTP foi desconectado ou não está disponível.".to_owned());
        }
        let point = self.mounts.mountpoint(uri)?;
        match (&operation, point) {
            (Operation::Mount(_), None) => self.mounts.mount(uri)?,
            (Operation::Unmount(_), Some(_)) => self.mounts.unmount(uri)?,
            _ => {}
        }
        self.with_mountpoints(self.devices.connected()?)
    }
}

impl<D: Phones, M: Mounts> MtpVolumes<D, M> {
    fn with_mountpoints(&self, devices: Vec<Volume>) -> Result<Vec<Volume>, String> {
        devices
            .into_iter()
            .map(|mut device| {
                device.mountpoint = self.mounts.mountpoint(&device.path)?;
                Ok(device)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
