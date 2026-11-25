use super::drill::{LEAK_GUARD, divide};
use super::encoding::{Sample, WINDOW};
use super::learning::lesson;
use super::reference::bases;
use crate::domain::{Activities, Features, Metric};

fn gapped() -> Vec<Sample> {
    (0..1200)
        .map(|index| Sample {
            at: index as f64 * 2.0,
            features: Features::default().with(
                Metric::CpuBusy,
                (!(250..600).contains(&index)).then_some(40.0 + (index % 7) as f32),
            ),
            activities: Activities::default(),
        })
        .collect()
}

#[test]
fn corte_cronologico_nas_linhas_brutas_com_folga() {
    let division = divide(1200);
    assert_eq!(division.split, 840);
    assert_eq!(division.last + LEAK_GUARD, division.split);
    assert_eq!(divide(10).last, WINDOW);
}

#[test]
fn treino_so_usa_origens_e_alvos_antes_da_avaliacao() {
    let rows = gapped();
    let bases = bases(&rows);
    let division = divide(rows.len());
    let lessons = (WINDOW..division.last)
        .filter(|end| lesson(&rows, &bases, division.split, *end).is_some())
        .count();
    assert!(lessons > 0);
    let leaked = (division.split..rows.len())
        .filter(|end| lesson(&rows, &bases, division.split, *end).is_some())
        .count();
    assert_eq!(leaked, 0);
}
