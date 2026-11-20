use std::time::{Duration, Instant};

use super::engine::{Block, Capabilities, Policy, Situation};
use crate::domain::{Adjustment, Authority, Change, Identity, IoClass};

const ALL: Capabilities = Capabilities {
    restore_nice: true,
    io_priority: true,
    affinity: true,
};

fn adjust(pid: u32, change: Change) -> Adjustment {
    Adjustment {
        target: Identity { pid, started: 100 },
        change,
    }
}

fn situation(authority: Authority, protected: &[u32], now: Instant) -> Situation<'_> {
    Situation {
        authority,
        telemetry: true,
        protected,
        capabilities: ALL,
        now,
    }
}

#[test]
fn sem_autoridade_nada_passa() {
    let now = Instant::now();
    let result = Policy::default().decide(
        &adjust(900, Change::Nice(10)),
        1.0,
        &situation(Authority::None, &[], now),
    );
    assert_eq!(result, Err(Block::NoAuthority));
}

#[test]
fn processos_protegidos_e_do_sistema_sao_intocaveis() {
    let now = Instant::now();
    let mut policy = Policy::default();
    let here = situation(Authority::Adaptive, &[1233, 4000], now);
    assert_eq!(
        policy.decide(&adjust(4000, Change::Nice(5)), 1.0, &here),
        Err(Block::Protected)
    );
    assert_eq!(
        policy.decide(&adjust(1, Change::Nice(5)), 1.0, &here),
        Err(Block::Protected)
    );
}

#[test]
fn baixa_confianca_ou_telemetria_ausente_bloqueiam() {
    let now = Instant::now();
    let mut policy = Policy::default();
    let mut here = situation(Authority::Adaptive, &[], now);
    assert_eq!(
        policy.decide(&adjust(900, Change::Nice(5)), 0.5, &here),
        Err(Block::LowConfidence)
    );
    assert_eq!(
        policy.decide(&adjust(900, Change::Nice(5)), f32::NAN, &here),
        Err(Block::LowConfidence)
    );
    here.telemetry = false;
    assert_eq!(
        policy.decide(&adjust(900, Change::Nice(5)), 1.0, &here),
        Err(Block::TelemetryMissing)
    );
}

#[test]
fn nunca_aumenta_prioridade_acima_do_padrao() {
    let now = Instant::now();
    let here = situation(Authority::Adaptive, &[], now);
    let mut policy = Policy::default();
    assert_eq!(
        policy.decide(&adjust(900, Change::Nice(-5)), 1.0, &here),
        Err(Block::OutOfRange)
    );
    assert_eq!(
        policy.decide(
            &adjust(900, Change::IoPriority(IoClass::BestEffort(9))),
            1.0,
            &here
        ),
        Err(Block::OutOfRange)
    );
    assert_eq!(
        policy.decide(&adjust(900, Change::Affinity(Vec::new())), 1.0, &here),
        Err(Block::OutOfRange)
    );
}

#[test]
fn sem_como_desfazer_o_ajuste_e_bloqueado() {
    let now = Instant::now();
    let mut here = situation(Authority::Adaptive, &[], now);
    here.capabilities = Capabilities {
        restore_nice: false,
        ..ALL
    };
    assert_eq!(
        Policy::default().decide(&adjust(900, Change::Nice(5)), 1.0, &here),
        Err(Block::NotReversible)
    );
}

#[test]
fn conservador_nao_mexe_em_afinidade() {
    let now = Instant::now();
    let here = situation(Authority::Conservative, &[], now);
    let mut policy = Policy::default();
    assert_eq!(
        policy.decide(&adjust(900, Change::Affinity(vec![0, 1])), 1.0, &here),
        Err(Block::Conservative)
    );
    assert_eq!(
        policy.decide(&adjust(900, Change::IoPriority(IoClass::Idle)), 1.0, &here),
        Ok(())
    );
}

#[test]
fn espera_entre_ajustes_do_mesmo_processo_e_limita_a_frequencia() {
    let start = Instant::now();
    let mut policy = Policy::default();
    let at = |seconds| {
        situation(
            Authority::Adaptive,
            &[],
            start + Duration::from_secs(seconds),
        )
    };
    assert_eq!(
        policy.decide(&adjust(900, Change::Nice(5)), 1.0, &at(0)),
        Ok(())
    );
    assert_eq!(
        policy.decide(&adjust(900, Change::Nice(6)), 1.0, &at(10)),
        Err(Block::Cooldown)
    );
    assert_eq!(
        policy.decide(&adjust(901, Change::Nice(5)), 1.0, &at(11)),
        Ok(())
    );
    assert_eq!(
        policy.decide(&adjust(902, Change::Nice(5)), 1.0, &at(12)),
        Ok(())
    );
    assert_eq!(
        policy.decide(&adjust(903, Change::Nice(5)), 1.0, &at(13)),
        Err(Block::RateLimit)
    );
    assert_eq!(
        policy.decide(&adjust(900, Change::Nice(6)), 1.0, &at(61)),
        Ok(())
    );
}
