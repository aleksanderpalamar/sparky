use std::fs;
use std::process::Command;

use super::root_fs::RootFs;
use super::signals::ProcessSignals;
use super::statvfs::Statvfs;
use crate::domain::process::parse_stat;
use crate::ports::{Filesystems, Identity, SignalError, Signals, Stop, SystemFiles};

#[test]
fn le_lista_e_segue_links_de_verdade() {
    let dir = tempfile::tempdir().unwrap();
    let raiz = dir.path().to_string_lossy().into_owned();
    fs::write(dir.path().join("cmdline"), b"/usr/bin/x\0-a\0").unwrap();
    std::os::unix::fs::symlink("/dev/dri/renderD128", dir.path().join("3")).unwrap();
    assert_eq!(
        RootFs.read(&format!("{raiz}/cmdline")).as_deref(),
        Some("/usr/bin/x\0-a\0")
    );
    let mut nomes = RootFs.list(&raiz);
    nomes.sort();
    assert_eq!(nomes, ["3", "cmdline"]);
    assert_eq!(
        RootFs.link(&format!("{raiz}/3")).as_deref(),
        Some("/dev/dri/renderD128")
    );
    assert_eq!(RootFs.read(&format!("{raiz}/sumiu")), None);
    assert!(RootFs.list(&format!("{raiz}/sumiu")).is_empty());
}

#[test]
fn capacidade_do_sistema_de_arquivos_tem_uso_menor_que_o_total() {
    let dir = tempfile::tempdir().unwrap();
    let (total, used) = Statvfs.usage(&dir.path().to_string_lossy()).unwrap();
    assert!(total > 0 && used <= total);
    assert_eq!(Statvfs.usage("/caminho/que/nao/existe"), None);
}

#[test]
fn trava_o_processo_pelo_pidfd_e_recusa_pid_com_outro_inicio() {
    let mut filho = Command::new("sleep").arg("30").spawn().unwrap();
    let stat = fs::read_to_string(format!("/proc/{}/stat", filho.id())).unwrap();
    let started = parse_stat(&stat).unwrap().started;
    let outro = Identity {
        pid: filho.id(),
        started: started + 1,
    };
    assert_eq!(ProcessSignals.claim(outro).err(), Some(SignalError::Gone));
    let identity = Identity {
        pid: filho.id(),
        started,
    };
    let trava = ProcessSignals.claim(identity).unwrap();
    assert_eq!(trava.send(Stop::Terminate), Ok(()));
    assert!(!filho.wait().unwrap().success());
    assert_eq!(trava.send(Stop::Kill), Err(SignalError::Gone));
}
