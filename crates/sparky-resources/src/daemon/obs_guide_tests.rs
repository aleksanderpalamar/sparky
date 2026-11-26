use std::time::{Duration, Instant};

use super::obs_guide::{Guide, Launch};
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
