use std::sync::Mutex;

use super::{Mounts, MtpVolumes, Phones};
use crate::{Operation, Volume, VolumeSource};

const URI: &str = "mtp://Android_Test/";
const POINT: &str = "/run/user/1000/gvfs/mtp:host=Android_Test";

fn phone() -> Volume {
    Volume {
        path: URI.to_owned(),
        label: "Android de teste".to_owned(),
        filesystem: "mtp".to_owned(),
        size: String::new(),
        mountpoint: None,
        removable: true,
    }
}

struct Devices(Vec<Volume>);

impl Phones for Devices {
    fn connected(&self) -> Result<Vec<Volume>, String> {
        Ok(self.0.clone())
    }
}

#[derive(Default)]
struct Backend {
    point: Mutex<Option<String>>,
    calls: Mutex<Vec<&'static str>>,
    failure: Option<String>,
}

impl Mounts for Backend {
    fn mountpoint(&self, _: &str) -> Result<Option<String>, String> {
        Ok(self.point.lock().unwrap().clone())
    }

    fn mount(&self, _: &str) -> Result<(), String> {
        self.calls.lock().unwrap().push("mount");
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        *self.point.lock().unwrap() = Some(POINT.to_owned());
        Ok(())
    }

    fn unmount(&self, _: &str) -> Result<(), String> {
        self.calls.lock().unwrap().push("unmount");
        *self.point.lock().unwrap() = None;
        Ok(())
    }
}

fn source() -> MtpVolumes<Devices, Backend> {
    MtpVolumes {
        devices: Devices(vec![phone()]),
        mounts: Backend::default(),
    }
}

#[test]
fn telefone_mtp_aparece_sem_dispositivo_de_bloco_e_sem_montagem_automatica() {
    let source = source();
    assert_eq!(source.run(Operation::List).unwrap(), vec![phone()]);
    assert!(source.mounts.calls.lock().unwrap().is_empty());
}

#[test]
fn montagem_e_desmontagem_preservam_a_identidade_do_telefone() {
    let source = source();
    let mounted = source.run(Operation::Mount(URI.to_owned())).unwrap();
    assert_eq!(mounted[0].path, URI);
    assert_eq!(mounted[0].mountpoint.as_deref(), Some(POINT));
    assert_eq!(
        source.run(Operation::Unmount(URI.to_owned())).unwrap(),
        vec![phone()]
    );
    assert_eq!(*source.mounts.calls.lock().unwrap(), ["mount", "unmount"]);
}

#[test]
fn telefone_ja_montado_reutiliza_o_acesso_existente() {
    let source = source();
    *source.mounts.point.lock().unwrap() = Some(POINT.to_owned());
    let volumes = source.run(Operation::Mount(URI.to_owned())).unwrap();
    assert_eq!(volumes[0].mountpoint.as_deref(), Some(POINT));
    assert!(source.mounts.calls.lock().unwrap().is_empty());
}

#[test]
fn dispositivo_removido_ou_uri_arbitraria_nao_dispara_montagem() {
    let source = source();
    for uri in ["mtp://ausente/", "file:///etc", "/dev/sda1"] {
        assert!(source.run(Operation::Mount(uri.to_owned())).is_err());
    }
    assert!(source.mounts.calls.lock().unwrap().is_empty());
    let empty = MtpVolumes {
        devices: Devices(Vec::new()),
        mounts: Backend::default(),
    };
    assert!(empty.run(Operation::Mount(URI.to_owned())).is_err());
}

#[test]
fn falha_do_backend_e_exposta_sem_transformar_o_telefone_em_montado() {
    let mut source = source();
    source.mounts.failure = Some("Desbloqueie o telefone".to_owned());
    assert_eq!(
        source.run(Operation::Mount(URI.to_owned())),
        Err("Desbloqueie o telefone".to_owned())
    );
    assert_eq!(source.run(Operation::List).unwrap(), vec![phone()]);
}
