use super::diagnosis::{ENTRY, Phase, SEGMENTS, Segment, Tally};
use super::encoding::{OUTPUTS, Sample};
use crate::domain::{Activities, Activity, Features};

fn at(seconds: f64, activities: &[Activity]) -> Sample {
    Sample {
        at: seconds,
        features: Features::default(),
        activities: activities.iter().copied().collect::<Activities>(),
    }
}

#[test]
fn separa_desktop_entrada_jogo_e_outros() {
    let mut phase = Phase::default();
    assert_eq!(
        phase.segment(&at(0.0, &[Activity::Desktop])),
        Segment::Desktop
    );
    assert_eq!(phase.segment(&at(2.0, &[Activity::Idle])), Segment::Desktop);
    assert_eq!(
        phase.segment(&at(4.0, &[Activity::Gaming])),
        Segment::Entering
    );
    assert_eq!(
        phase.segment(&at(4.0 + ENTRY - 1.0, &[Activity::Gaming])),
        Segment::Entering
    );
    assert_eq!(
        phase.segment(&at(4.0 + ENTRY, &[Activity::Gaming, Activity::Recording])),
        Segment::Playing
    );
    assert_eq!(
        phase.segment(&at(200.0, &[Activity::Compiling])),
        Segment::Other
    );
}

#[test]
fn sair_do_jogo_reinicia_a_entrada() {
    let mut phase = Phase::default();
    phase.segment(&at(0.0, &[Activity::Gaming]));
    assert_eq!(
        phase.segment(&at(90.0, &[Activity::Gaming])),
        Segment::Playing
    );
    phase.segment(&at(92.0, &[Activity::Desktop]));
    assert_eq!(
        phase.segment(&at(94.0, &[Activity::Gaming])),
        Segment::Entering
    );
}

#[test]
fn conta_e_soma_por_grupo_e_saida() {
    let mut tally = Tally::default();
    tally.add(Segment::Playing, 0, 2.0, 4.0);
    tally.add(Segment::Playing, 0, 1.0, 2.0);
    tally.add(Segment::Desktop, 7, 0.5, 0.5);
    let cell = tally.cell(Segment::Playing, 0);
    assert_eq!((cell.count, cell.neural, cell.statistical), (2, 3.0, 6.0));
    assert_eq!(tally.cell(Segment::Entering, 0).count, 0);
    let errors = tally.errors(true);
    assert_eq!(errors.count, 3);
    assert_eq!(errors.neural, Some(3.5 / 3.0));
    assert_eq!(errors.statistical, 6.5 / 3.0);
    assert_eq!(tally.errors(false).neural, None);
}

#[test]
fn as_linhas_trazem_grupo_metrica_horizonte_e_contagem() {
    let mut tally = Tally::default();
    tally.add(Segment::Playing, 0, 2.0, 4.0);
    tally.add(Segment::Entering, OUTPUTS - 1, 1.0, 3.0);
    assert_eq!(
        tally.lines(),
        vec![
            "entrada VramUsed 60s n=1 rede=1.00 estat=3.00".to_owned(),
            "jogo CpuBusy 10s n=1 rede=2.00 estat=4.00".to_owned(),
        ]
    );
    assert_eq!(SEGMENTS.len(), 4);
}
