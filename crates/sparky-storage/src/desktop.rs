use crate::mtp::MtpVolumes;
use crate::mtp::gio_mounts::GioMounts;
use crate::mtp::udev_phones::UdevPhones;
use crate::system::{SystemCommands, SystemVolumes};
use crate::{Operation, Volume, VolumeKind, VolumeSource};

pub struct DesktopVolumes<B, M> {
    pub disks: B,
    pub mobile: M,
}

pub fn system() -> impl VolumeSource {
    DesktopVolumes {
        disks: SystemVolumes(SystemCommands),
        mobile: MtpVolumes {
            devices: UdevPhones,
            mounts: GioMounts,
        },
    }
}

impl<B: VolumeSource, M: VolumeSource> VolumeSource for DesktopVolumes<B, M> {
    fn run(&self, operation: Operation) -> Result<Vec<Volume>, String> {
        let (disk_operation, mobile_operation) = match &operation {
            Operation::Mount(path) | Operation::Unmount(path)
                if VolumeKind::for_path(path) == VolumeKind::Mtp =>
            {
                (Operation::List, operation)
            }
            _ => (operation, Operation::List),
        };
        let mut volumes = self.disks.run(disk_operation)?;
        volumes.extend(self.mobile.run(mobile_operation)?);
        Ok(volumes)
    }
}

#[cfg(test)]
mod tests;
