use std::collections::VecDeque;
use std::sync::Mutex;

use crate::system::{Commands, LIST_ARGUMENTS, SystemVolumes};
use crate::{Operation, VolumeSource};

const USB: &str = r#"{"blockdevices":[{"path":"/dev/sda1","type":"part","fstype":"vfat","label":"USB","rm":true}]}"#;
const MOUNTED: &str = r#"{"blockdevices":[{"path":"/dev/sda1","type":"part","fstype":"vfat","label":"USB","rm":true,"mountpoint":"/run/media/p/USB"}]}"#;

struct Fake {
    answers: Mutex<VecDeque<Result<String, String>>>,
    calls: Mutex<Vec<(String, Vec<String>)>>,
}

impl Fake {
    fn new(answers: &[Result<&str, &str>]) -> Self {
        Self {
            answers: Mutex::new(
                answers
                    .iter()
                    .map(|answer| answer.map(str::to_owned).map_err(str::to_owned))
                    .collect(),
            ),
            calls: Mutex::default(),
        }
    }
}

impl Commands for Fake {
    fn output(&self, program: &str, arguments: &[&str]) -> Result<String, String> {
        self.calls.lock().map_err(|error| error.to_string())?.push((
            program.to_owned(),
            arguments.iter().map(|arg| (*arg).to_owned()).collect(),
        ));
        self.answers
            .lock()
            .map_err(|error| error.to_string())?
            .pop_front()
            .ok_or("chamada inesperada")?
    }
}

#[test]
fn montar_reconsulta_e_retorna_o_ponto_de_montagem_real() {
    let source = SystemVolumes(Fake::new(&[Ok(USB), Ok("Mounted"), Ok(MOUNTED)]));
    let result = source.run(Operation::Mount("/dev/sda1".to_owned())).unwrap_or_default();
    assert_eq!(result[0].mountpoint.as_deref(), Some("/run/media/p/USB"));
    let calls = source.0.calls.lock().unwrap();
    assert_eq!(
        calls[0],
        (
            "lsblk".to_owned(),
            LIST_ARGUMENTS.iter().map(|arg| (*arg).to_owned()).collect()
        )
    );
    assert_eq!(
        calls[1],
        (
            "udisksctl".to_owned(),
            vec!["mount".to_owned(), "-b".to_owned(), "/dev/sda1".to_owned()]
        )
    );
    assert_eq!(calls.len(), 3);
}

#[test]
fn desmontar_nao_forca_e_reconsulta_o_estado() {
    let source = SystemVolumes(Fake::new(&[Ok(MOUNTED), Ok("Unmounted"), Ok(USB)]));
    let result = source
        .run(Operation::Unmount("/dev/sda1".to_owned()))
        .unwrap_or_default();
    assert!(!result[0].mounted());
    assert_eq!(source.0.calls.lock().unwrap()[1].1, ["unmount", "-b", "/dev/sda1"]);
}

#[test]
fn volume_removido_e_sistema_protegido_nao_disparam_udisksctl() {
    for device in ["/dev/desapareceu", "/dev/root", "--help"] {
        let source = SystemVolumes(Fake::new(&[Ok(USB)]));
        assert!(source.run(Operation::Mount(device.to_owned())).is_err());
        assert_eq!(source.0.calls.lock().unwrap().len(), 1);
    }
}

#[test]
fn recusa_de_permissao_e_resposta_invalida_sao_preservadas() {
    let source = SystemVolumes(Fake::new(&[Ok(USB), Err("Not authorized")]));
    assert_eq!(
        source.run(Operation::Mount("/dev/sda1".to_owned())),
        Err("Not authorized".to_owned())
    );
    let invalid = SystemVolumes(Fake::new(&[Ok("invalido")]));
    assert!(
        invalid
            .run(Operation::List)
            .is_err_and(|error| error.contains("resposta inválida"))
    );
}
