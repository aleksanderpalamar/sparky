use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use super::elevator::{finished, verdict};
use crate::elevation::Refusal;
use crate::ports::ExecError;

#[test]
fn traduz_o_codigo_de_saida_do_pkexec_e_do_helper() {
    assert_eq!(verdict(Some(0)), Ok(()));
    assert_eq!(verdict(Some(126)), Err(ExecError::Denied));
    assert_eq!(verdict(Some(127)), Err(ExecError::Denied));
    let gone = i32::from(Refusal::Gone.code());
    assert_eq!(verdict(Some(gone)), Err(ExecError::Gone));
    for refusal in [Refusal::OutOfRange, Refusal::NotOwner, Refusal::Protected] {
        assert_eq!(
            verdict(Some(i32::from(refusal.code()))),
            Err(ExecError::Denied)
        );
    }
    assert_eq!(verdict(Some(1)), Err(ExecError::Failed));
    assert_eq!(verdict(None), Err(ExecError::Failed));
}

#[test]
fn processo_preso_e_encerrado_no_limite_de_tempo() {
    let Ok(child) = Command::new("sleep")
        .arg("30")
        .stdout(Stdio::null())
        .spawn()
    else {
        return;
    };
    let start = Instant::now();
    assert_eq!(finished(child, Duration::from_millis(200)), None);
    assert!(start.elapsed() < Duration::from_secs(5));
}

#[test]
fn devolve_o_codigo_de_quem_termina_a_tempo() {
    let Ok(child) = Command::new("sh").args(["-c", "exit 68"]).spawn() else {
        return;
    };
    assert_eq!(finished(child, Duration::from_secs(5)), Some(68));
}
