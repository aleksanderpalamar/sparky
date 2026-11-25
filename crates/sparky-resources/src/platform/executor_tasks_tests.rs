use std::io::ErrorKind;

use super::executor::LinuxExecutor;
use crate::domain::Identity;
use crate::ports::{Elevator, ExecError, Executor};
use crate::telemetry::fake_files::FakeFiles;

const STAT: &str = "/proc/900/stat";
const TASKS: &str = "/proc/900/task";
const ALIVE: &str = "900 (cat) S 1 0 0 0 0 0 0 0 0 0 5 5 0 0 0 0 0 0 777";

struct NoElevator;

impl Elevator for NoElevator {
    fn renice(&self, _task: u32, _value: i8) -> Result<(), ExecError> {
        Err(ExecError::Denied)
    }
}

fn target() -> Identity {
    Identity {
        pid: 900,
        started: 777,
    }
}

fn tasks(files: FakeFiles) -> Result<Vec<u32>, ExecError> {
    LinuxExecutor::new(files, NoElevator).tasks(target())
}

fn alive() -> FakeFiles {
    FakeFiles::default()
        .with(STAT, ALIVE)
        .with("/proc/900/task/900/stat", "")
        .with("/proc/900/task/901/stat", "")
}

#[test]
fn processo_vivo_devolve_as_threads() {
    assert_eq!(tasks(alive()), Ok(vec![900, 901]));
}

#[test]
fn processo_inexistente_foi_encerrado() {
    assert_eq!(tasks(FakeFiles::default()), Err(ExecError::Gone));
}

#[test]
fn pid_reutilizado_por_outro_processo_foi_encerrado() {
    let files = FakeFiles::default().with(STAT, &ALIVE.replace(" 777", " 778"));
    assert_eq!(tasks(files), Err(ExecError::Gone));
}

#[test]
fn permissao_negada_nao_e_encerramento() {
    let files = FakeFiles::default().broken(STAT, ErrorKind::PermissionDenied);
    assert_eq!(tasks(files), Err(ExecError::Denied));
}

#[test]
fn erro_de_leitura_nao_e_encerramento() {
    let files = FakeFiles::default().broken(STAT, ErrorKind::Other);
    assert_eq!(tasks(files), Err(ExecError::Failed));
    let files = alive().broken(TASKS, ErrorKind::Other);
    assert_eq!(tasks(files), Err(ExecError::Failed));
}

#[test]
fn stat_ilegivel_nao_e_encerramento() {
    let files = FakeFiles::default().with(STAT, "lixo");
    assert_eq!(tasks(files), Err(ExecError::Failed));
}
