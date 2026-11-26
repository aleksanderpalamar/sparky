use std::time::{Duration, Instant};

use super::obs_guide::{Guide, Launch, spawn};
use crate::context::{Sensed, Unavailable};

#[derive(Default)]
struct Counter(usize);

impl Launch for Counter {
    fn launch(&mut self) {
        self.0 += 1;
    }
}

const DISABLED: Sensed = Sensed::Unavailable(Unavailable::Disabled);
const UNREACHABLE: Sensed = Sensed::Unavailable(Unavailable::Unreachable);

fn guide() -> Guide<Counter> {
    Guide::new(Counter::default())
}

#[test]
fn servidor_desligado_avisa_uma_vez_por_abertura_do_obs() {
    let (mut guide, start) = (guide(), Instant::now());
    guide.consider(Some((900, DISABLED)), start, || true);
    guide.consider(
        Some((900, DISABLED)),
        start + Duration::from_secs(2),
        || true,
    );
    assert_eq!(guide.launcher().0, 1);
    guide.consider(
        Some((901, DISABLED)),
        start + Duration::from_secs(9),
        || true,
    );
    assert_eq!(guide.launcher().0, 2);
}

#[test]
fn servidor_inacessivel_espera_o_obs_terminar_de_abrir() {
    let (mut guide, start) = (guide(), Instant::now());
    guide.consider(Some((900, UNREACHABLE)), start, || true);
    guide.consider(
        Some((900, UNREACHABLE)),
        start + Duration::from_secs(20),
        || true,
    );
    assert_eq!(guide.launcher().0, 0);
    guide.consider(
        Some((900, UNREACHABLE)),
        start + Duration::from_secs(31),
        || true,
    );
    assert_eq!(guide.launcher().0, 1);
}

#[test]
fn preferencia_desligada_nao_avisa_e_nao_pergunta_de_novo() {
    let (mut guide, start) = (guide(), Instant::now());
    let mut asked = 0;
    for tick in 0..3 {
        guide.consider(
            Some((900, DISABLED)),
            start + Duration::from_secs(tick),
            || {
                asked += 1;
                false
            },
        );
    }
    assert_eq!(guide.launcher().0, 0);
    assert_eq!(asked, 1);
}

#[test]
fn obs_funcionando_fechado_ou_com_senha_recusada_nao_avisa() {
    let (mut guide, start) = (guide(), Instant::now());
    for sensed in [
        Some((900, Sensed::Idle)),
        Some((900, Sensed::Capturing)),
        Some((900, Sensed::Unavailable(Unavailable::Unauthorized))),
        None,
    ] {
        guide.consider(sensed, start + Duration::from_secs(60), || true);
    }
    assert_eq!(guide.launcher().0, 0);
}

#[test]
fn o_aviso_nasce_sem_os_sinais_bloqueados_do_daemon() {
    use nix::sys::signal::{SigSet, Signal};
    use std::process::{Command, Stdio};
    let mut blocked = SigSet::empty();
    blocked.add(Signal::SIGTERM);
    assert!(blocked.thread_block().is_ok());
    let mut command = Command::new("sh");
    command
        .args(["-c", "grep SigBlk /proc/self/status"])
        .stdout(Stdio::piped());
    let output = spawn(&mut command).and_then(|child| child.wait_with_output());
    assert!(blocked.thread_unblock().is_ok());
    let text = output.map(|output| String::from_utf8_lossy(&output.stdout).into_owned());
    assert!(
        text.as_deref()
            .is_ok_and(|text| text.contains("0000000000000000")),
        "{text:?}"
    );
}
