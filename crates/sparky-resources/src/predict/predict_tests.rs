use super::anomaly::Deviation;
use super::evaluation::{Persistence, Point, evaluate};
use super::holt::Holt;

fn ramp(count: usize, step: f64, slope: f32) -> Vec<Point> {
    (0..count)
        .map(|index| Point {
            at: index as f64 * step,
            value: 10.0 + slope * (index as f64 * step) as f32,
        })
        .collect()
}

#[test]
fn holt_acompanha_uma_tendencia_linear() {
    let mut holt = Holt::new(0.5, 0.3);
    for point in ramp(200, 2.0, 0.1) {
        holt.update(point.at, point.value);
    }
    let predicted = holt.forecast(10.0).unwrap_or_default();
    assert!(
        (predicted - (10.0 + 0.1 * 408.0)).abs() < 0.05,
        "{predicted}"
    );
}

#[test]
fn holt_respeita_intervalos_irregulares() {
    let mut holt = Holt::new(0.5, 0.3);
    let mut at = 0.0;
    for gap in [1.0, 5.0, 2.0, 0.25, 2.0].iter().cycle().take(300) {
        at += gap;
        holt.update(at, 50.0 + 0.2 * at as f32);
    }
    let predicted = holt.forecast(4.0).unwrap_or_default();
    assert!(
        (predicted - (50.0 + 0.2 * (at + 4.0) as f32)).abs() < 0.5,
        "{predicted}"
    );
}

#[test]
fn holt_ignora_amostra_repetida_ou_fora_de_ordem() {
    let mut holt = Holt::new(0.5, 0.3);
    holt.update(10.0, 20.0);
    holt.update(10.0, 90.0);
    holt.update(5.0, 90.0);
    assert_eq!(holt.forecast(0.0), Some(20.0));
    assert_eq!(Holt::new(0.5, 0.3).forecast(1.0), None);
}

#[test]
fn desvio_so_pontua_depois_do_aquecimento() {
    let mut deviation = Deviation::new(0.1);
    assert_eq!(deviation.score(50.0), None);
    for index in 0..60 {
        deviation.update(40.0 + (index % 3) as f32);
    }
    assert!(deviation.score(41.0).is_some_and(|score| score < 1.5));
    assert!(deviation.score(95.0).is_some_and(|score| score > 10.0));
}

#[test]
fn tendencia_vence_a_persistencia_em_crescimento_constante() {
    let points = ramp(300, 2.0, 0.05);
    let holt = evaluate(&points, &mut Holt::new(0.5, 0.3), 10.0);
    let naive = evaluate(&points, &mut Persistence::default(), 10.0);
    let (Some(holt), Some(naive)) = (holt, naive) else {
        panic!("as duas avaliações deveriam produzir previsões");
    };
    assert!((naive.mean_absolute_error - 0.5).abs() < 0.01, "{naive:?}");
    assert!(
        holt.mean_absolute_error < naive.mean_absolute_error / 4.0,
        "{holt:?}"
    );
    assert_eq!(holt.forecasts, naive.forecasts);
}

#[test]
fn lacunas_maiores_que_a_tolerancia_nao_viram_previsao() {
    let points = [
        Point {
            at: 0.0,
            value: 1.0,
        },
        Point {
            at: 100.0,
            value: 2.0,
        },
    ];
    assert_eq!(evaluate(&points, &mut Persistence::default(), 10.0), None);
}
