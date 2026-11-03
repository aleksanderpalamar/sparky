use std::os::unix::process::ExitStatusExt;
use std::process::{Child, Command};
use std::time::Duration;

use tokio::sync::oneshot;

use crate::service::greeter_process::GreeterProcess;

fn outro_processo() -> Child {
    Command::new("sleep")
        .arg("30")
        .spawn()
        .unwrap_or_else(|error| panic!("{error}"))
}

fn ainda_vivo(child: &mut Child) -> bool {
    std::thread::sleep(Duration::from_millis(100));
    matches!(child.try_wait(), Ok(None))
}

#[test]
fn depois_que_a_tela_fecha_o_pid_antigo_nao_recebe_sinal() {
    let mut reaproveitado = outro_processo();
    let mut greeter = GreeterProcess::default();
    let (stop, _stopped) = oneshot::channel();
    greeter.started(stop, Some(reaproveitado.id()));
    greeter.gone();
    greeter.terminate();
    assert!(ainda_vivo(&mut reaproveitado));
    let _ = reaproveitado.kill();
    let _ = reaproveitado.wait();
}

#[test]
fn com_a_tela_aberta_encerrar_manda_sigterm() {
    let mut tela = outro_processo();
    let mut greeter = GreeterProcess::default();
    let (stop, _stopped) = oneshot::channel();
    greeter.started(stop, Some(tela.id()));
    greeter.terminate();
    let status = tela.wait().unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(status.signal(), Some(libc::SIGTERM));
}

#[test]
fn pedir_para_fechar_avisa_quem_acompanha_a_tela() {
    let mut greeter = GreeterProcess::default();
    let (stop, mut stopped) = oneshot::channel();
    greeter.started(stop, None);
    greeter.ask_to_stop();
    assert_eq!(stopped.try_recv(), Ok(()));
}
