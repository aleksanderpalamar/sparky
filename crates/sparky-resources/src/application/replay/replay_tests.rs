use super::confirm::confirm;
use super::report::summarize;
use super::resample::interval;
use super::sessions::{overlap, reserve, sessions};
use crate::application::archive::GAP;
use crate::application::encoding::{Sample, WINDOW};
use crate::domain::{Activities, Activity, Features, Metric};

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

fn three_sessions() -> Vec<Sample> {
    let mut rows = wave(0.0, 300, Activity::Desktop);
    let next = rows.last().map_or(0.0, |last| last.at) + GAP;
    rows.extend(wave(next, 300, Activity::Compiling));
    let next = rows.last().map_or(0.0, |last| last.at) + GAP;
    rows.extend(wave(next, 300, Activity::Recording));
    rows
}

#[test]
fn sessoes_sao_separadas_pela_lacuna() {
    assert_eq!(
        sessions(&three_sessions()),
        vec![0..300, 300..600, 600..900]
    );
    assert!(sessions(&[]).is_empty());
}

#[test]
fn reserva_sessoes_inteiras_no_fim_e_nunca_as_entrega_ao_treino() {
    let rows = three_sessions();
    let Some(holdout) = reserve(&rows, 0.3) else {
        panic!("deveria reservar")
    };
    assert_eq!(holdout.training, 0..600);
    assert_eq!(holdout.reserved, 600..900);
    assert!(holdout.training.end <= holdout.reserved.start);
}

#[test]
fn sessao_unica_reserva_o_bloco_mais_recente() {
    let rows = wave(0.0, 1000, Activity::Desktop);
    let Some(holdout) = reserve(&rows, 0.3) else {
        panic!("deveria reservar")
    };
    assert_eq!(holdout.reserved, 700..1000);
    assert!(reserve(&rows[..WINDOW], 0.3).is_none());
}

#[test]
fn sobreposicao_entre_blocos() {
    assert_eq!(overlap(&(0..100), &(50..150)), 0.5);
    assert_eq!(overlap(&(0..100), &(100..200)), 0.0);
}

#[test]
fn confirmacao_so_usa_origens_reservadas_e_separa_por_atividade() {
    let rows = three_sessions();
    let pairs = confirm(&rows, 600..900, None);
    assert!(!pairs.is_empty());
    let first = rows[600].at;
    assert!(pairs.values().flatten().all(|pair| pair.at >= first));
    assert!(
        pairs
            .keys()
            .all(|(segment, _)| segment.label() == "gravacao")
    );
    assert!(pairs.values().flatten().all(|pair| pair.neural.is_none()));
}

#[test]
fn reamostragem_em_blocos_e_deterministica_e_cobre_a_media() {
    let values: Vec<f64> = (0..200).map(|index| (index % 10) as f64).collect();
    let (Some(first), Some(again)) = (interval(&values, 20, 500, 7), interval(&values, 20, 500, 7))
    else {
        panic!("deveria estimar")
    };
    assert_eq!(first, again);
    assert!(first.low <= first.mean && first.mean <= first.high);
    assert!(interval(&[], 20, 500, 7).is_none());
}

#[test]
fn resumo_pareado_aponta_regressao_quando_a_rede_e_pior() {
    let rows = three_sessions();
    let pairs = confirm(&rows, 600..900, None);
    let Some(cut) = pairs.values().next() else {
        panic!("deveria haver pares")
    };
    let worse: Vec<_> = cut
        .iter()
        .map(|pair| super::confirm::Paired {
            neural: Some(pair.statistical + 5.0),
            ..*pair
        })
        .collect();
    let summary = summarize(&worse, 7);
    assert_eq!(summary.count, worse.len());
    assert!(summary.regression);
    assert!(summary.margin.is_some_and(|margin| margin < 0.0));
}

fn paired(count: usize, gain: f64) -> Vec<super::confirm::Paired> {
    (0..count)
        .map(|index| super::confirm::Paired {
            at: index as f64 * 2.0,
            neural: Some(1.0 - gain),
            statistical: 1.0,
            persistence: Some(1.0),
        })
        .collect()
}

#[test]
fn poucas_amostras_nao_tem_intervalo_nem_alerta() {
    let summary = summarize(&paired(24, -0.5), 7);
    assert!(summary.gain.is_none());
    assert!(!summary.regression);
    assert!(summary.margin.is_some_and(|margin| margin < 0.0));
}

#[test]
fn perda_insignificante_nao_e_regressao() {
    assert!(!summarize(&paired(300, -0.001), 7).regression);
    assert!(summarize(&paired(300, -0.5), 7).regression);
}

#[test]
fn coletas_encadeadas_viram_sessoes_separadas() {
    let rows = super::holdout::chain(vec![
        wave(0.0, 50, Activity::Desktop),
        wave(0.0, 50, Activity::Desktop),
    ]);
    assert_eq!(sessions(&rows), vec![0..50, 50..100]);
}

#[test]
fn treino_congelado_e_deterministico_e_so_ve_o_treino() {
    use crate::application::learning_support::small;
    let rows = wave(0.0, 400, Activity::Desktop);
    let first = super::holdout::freeze(small(), &rows[..320]).model;
    let again = super::holdout::freeze(small(), &rows[..320]).model;
    assert_eq!(first, again);
    let mut changed = rows.clone();
    for sample in &mut changed[320..] {
        sample.features = Features::default().with(Metric::CpuBusy, Some(99.0));
    }
    let blind = super::holdout::freeze(small(), &changed[..320]).model;
    assert_eq!(first, blind);
}
