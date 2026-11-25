use super::archive::GAP;
use super::encoding::{Sample, WINDOW, example_at};
use super::reference::bases;
use crate::domain::{Activities, Features, Metric};

const FIRST: usize = 120;

fn sample(at: f64, cpu: f32) -> Sample {
    Sample {
        at,
        features: Features::default().with(Metric::CpuBusy, Some(cpu)),
        activities: Activities::default(),
    }
}

fn two_sessions() -> Vec<Sample> {
    let before = (0..FIRST).map(|index| sample(index as f64 * 2.0, 100.0));
    let resumed = (FIRST as f64 - 1.0) * 2.0 + GAP;
    let after = (0..120).map(move |index| sample(resumed + index as f64 * 2.0, 0.0));
    before.chain(after).collect()
}

#[test]
fn referencia_recomeca_no_corte_como_um_daemon_novo() {
    let rows = two_sessions();
    let joined = bases(&rows);
    let alone = bases(&rows[FIRST..]);
    for offset in 0..20 {
        assert_eq!(joined[FIRST + offset], alone[offset], "instante {offset}");
    }
}

#[test]
fn nenhuma_janela_atravessa_o_corte() {
    let rows = two_sessions();
    let bases = bases(&rows);
    for end in FIRST + 1..FIRST + WINDOW {
        assert!(example_at(&rows, &bases, end).is_none(), "fim {end}");
    }
    assert!(example_at(&rows, &bases, FIRST + WINDOW).is_some());
}

#[test]
fn cada_sessao_continua_aproveitavel() {
    let rows = two_sessions();
    let bases = bases(&rows);
    let usable = (WINDOW..FIRST)
        .filter(|end| example_at(&rows, &bases, *end).is_some())
        .count();
    assert!(usable > 0);
}
