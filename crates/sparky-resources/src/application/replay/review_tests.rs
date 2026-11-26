use super::confirm::{Paired, confirm};
use super::holdout::freeze;
use super::report::summarize;
use super::shadow::fresh;
use crate::application::archive::GAP;
use crate::application::diagnosis::Segment;
use crate::application::encoding::Sample;
use crate::application::learning_support::small;
use crate::domain::{Activities, Activity, Features, Metric};
use crate::orchestration::{Decision, Purpose, Verdict};

fn sample(at: f64, cpu: f32, activity: Activity) -> Sample {
    Sample {
        at,
        features: Features::default().with(Metric::CpuBusy, Some(cpu)),
        activities: [activity].into_iter().collect::<Activities>(),
    }
}

fn wave(start: f64, count: usize, activity: Activity) -> Vec<Sample> {
    (0..count)
        .map(|index| {
            let at = start + index as f64 * 2.0;
            let cpu = 50.0 + 30.0 * ((at / 40.0) * std::f64::consts::TAU).sin() as f32;
            sample(at, cpu, activity)
        })
        .collect()
}

fn two_games() -> Vec<Sample> {
    let mut rows = wave(0.0, 300, Activity::Gaming);
    let next = rows.last().map_or(0.0, |last| last.at) + GAP;
    rows.extend(wave(next, 300, Activity::Gaming));
    rows
}

#[test]
fn saidas_do_mesmo_instante_sao_reamostradas_juntas() {
    let series: Vec<Paired> = (0..120)
        .map(|index| Paired {
            at: index as f64 * 2.0,
            neural: Some(if index % 2 == 0 { 0.5 } else { 1.5 }),
            statistical: 1.0,
            persistence: None,
        })
        .collect();
    let repeated: Vec<Paired> = (0..14).flat_map(|_| series.iter().copied()).collect();
    assert_eq!(summarize(&repeated, 7).gain, summarize(&series, 7).gain);
}

#[test]
fn a_entrada_no_jogo_recomeca_em_cada_sessao() {
    let rows = two_games();
    let cuts = confirm(&rows, 300..600, None);
    assert!(
        cuts.keys()
            .any(|(segment, _)| *segment == Segment::Entering)
    );
}

#[test]
fn confirmacao_respeita_o_fim_do_bloco_reservado() {
    let rows = wave(0.0, 1000, Activity::Desktop);
    let (first, last) = (rows[600].at, rows[699].at);
    let cuts = confirm(&rows, 600..700, None);
    assert!(!cuts.is_empty());
    assert!(
        cuts.values()
            .flatten()
            .all(|pair| pair.at >= first && pair.at <= last)
    );
    assert!(confirm(&rows, 600..600, None).is_empty());
}

#[test]
fn passos_contados_sao_os_aplicados_de_verdade() {
    let plan = small();
    let silent: Vec<Sample> = (0..plan.minimum + 50)
        .map(|index| Sample {
            at: index as f64 * 2.0,
            features: Features::default(),
            activities: Activities::default(),
        })
        .collect();
    assert_eq!(freeze(plan, &silent).updates, 0);
    let rows = wave(0.0, plan.capacity * 2, Activity::Desktop);
    let frozen = freeze(plan, &rows);
    let bound = plan.epochs * plan.capacity.div_ceil(plan.batch);
    assert!(frozen.updates > 0 && frozen.updates <= bound);
}

fn decision(at_seconds: u64) -> Decision {
    Decision {
        at_seconds,
        activities: Activities::default(),
        purpose: Purpose::EaseCpu,
        process: "rustc".to_owned(),
        change: crate::domain::Change::Nice(10),
        stall: None,
        verdict: Verdict::Applied,
    }
}

#[test]
fn decisoes_novas_sao_lidas_antes_de_sair_do_registro() {
    let entries: Vec<Decision> = (50..150).map(decision).collect();
    let new = fresh(&entries, 120, 150);
    assert_eq!(new.len(), 30);
    assert_eq!(new.first().map(|decision| decision.at_seconds), Some(120));
    assert!(fresh(&entries, 150, 150).is_empty());
}
