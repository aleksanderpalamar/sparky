use std::sync::Mutex;

use super::DesktopVolumes;
use crate::{Operation, Volume, VolumeKind, VolumeSource};

struct Source {
    path: &'static str,
    calls: Mutex<Vec<Operation>>,
}

impl Source {
    fn new(path: &'static str) -> Self {
        Self {
            path,
            calls: Mutex::default(),
        }
    }
}

impl VolumeSource for Source {
    fn run(&self, operation: Operation) -> Result<Vec<Volume>, String> {
        self.calls.lock().unwrap().push(operation);
        Ok(vec![Volume {
            path: self.path.to_owned(),
            size: String::new(),
            label: String::new(),
            filesystem: String::new(),
            mountpoint: None,
            removable: true,
        }])
    }
}

#[test]
fn lateral_combina_discos_e_telefones_sem_montar_nenhum_deles() {
    let source = DesktopVolumes {
        disks: Source::new("/dev/sda1"),
        mobile: Source::new("mtp://Android/"),
    };
    let volumes = source.run(Operation::List).unwrap();
    assert_eq!(volumes.len(), 2);
    assert_eq!(volumes[0].kind(), VolumeKind::Block);
    assert_eq!(volumes[1].kind(), VolumeKind::Mtp);
    assert_eq!(*source.disks.calls.lock().unwrap(), [Operation::List]);
    assert_eq!(*source.mobile.calls.lock().unwrap(), [Operation::List]);
}

#[test]
fn operacoes_mtp_nunca_seguem_para_o_backend_de_blocos() {
    for operation in [
        Operation::Mount("mtp://Android/".to_owned()),
        Operation::Unmount("mtp://Android/".to_owned()),
    ] {
        let source = DesktopVolumes {
            disks: Source::new("/dev/sda1"),
            mobile: Source::new("mtp://Android/"),
        };
        source.run(operation.clone()).unwrap();
        assert_eq!(*source.disks.calls.lock().unwrap(), [Operation::List]);
        assert_eq!(*source.mobile.calls.lock().unwrap(), [operation]);
    }
}

#[test]
fn operacoes_de_disco_preservam_o_backend_existente() {
    let source = DesktopVolumes {
        disks: Source::new("/dev/sda1"),
        mobile: Source::new("mtp://Android/"),
    };
    let operation = Operation::Mount("/dev/sda1".to_owned());
    source.run(operation.clone()).unwrap();
    assert_eq!(*source.disks.calls.lock().unwrap(), [operation]);
    assert_eq!(*source.mobile.calls.lock().unwrap(), [Operation::List]);
}
