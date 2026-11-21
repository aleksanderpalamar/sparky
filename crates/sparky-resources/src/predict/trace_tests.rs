use super::trace::{HEADER, line, parse, series};
use crate::domain::{Features, Metric};

#[test]
fn a_linha_vai_e_volta_com_lacunas() {
    let features = Features::default()
        .with(Metric::CpuBusy, Some(12.5))
        .with(Metric::VramUsed, Some(40.0));
    let text = line(2.0, &features);
    assert_eq!(text, "2.000,12.500,,,,,,40.000");
    assert_eq!(parse(&text), Some((2.0, features)));
    assert_eq!(HEADER.split(',').count(), text.split(',').count());
}

#[test]
fn linha_com_colunas_a_mais_a_menos_ou_invalidas_e_recusada() {
    assert_eq!(parse(HEADER), None);
    assert_eq!(parse("1.0,1,2,3"), None);
    assert_eq!(parse("1.0,1,2,3,4,5,6,7,8"), None);
    assert_eq!(parse("1.0,x,2,3,4,5,6,7"), None);
}

#[test]
fn a_serie_de_uma_metrica_pula_as_ausencias() {
    let rows = [
        (0.0, Features::default().with(Metric::CpuBusy, Some(1.0))),
        (2.0, Features::default()),
        (4.0, Features::default().with(Metric::CpuBusy, Some(3.0))),
    ];
    let points = series(&rows, Metric::CpuBusy);
    assert_eq!(
        points.iter().map(|point| point.at).collect::<Vec<_>>(),
        vec![0.0, 4.0]
    );
}
