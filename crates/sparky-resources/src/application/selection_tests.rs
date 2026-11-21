use super::encoding::Sample;
use super::learning::{Plan, shape};
use super::progress::Outcome;
use super::selection::{Judged, best, judge};
use super::validation::validate;
use crate::domain::{Activities, Features, Metric};
use crate::neural::Hybrid;

fn flat() -> Vec<Sample> {
    (0..300)
        .map(|index| Sample {
            at: index as f64 * 2.0,
            features: Features::default().with(Metric::CpuBusy, Some(30.0)),
            activities: Activities::default(),
        })
        .collect()
}

fn plan() -> Plan {
    Plan {
        hidden: 4,
        blend: 4,
        ..Plan::default()
    }
}

#[test]
fn sem_candidato_anterior_fica_o_novo() {
    let rows = flat();
    let fresh = Hybrid::new(shape(&plan()), 3);
    let (kept, _) = best(None, fresh.clone(), &rows, 210);
    assert_eq!(kept, fresh);
}

#[test]
fn fica_o_candidato_que_erra_menos_nos_mesmos_dados() {
    let rows = flat();
    let steady = Hybrid::still(shape(&plan()), 3);
    let noisy = Hybrid::new(shape(&plan()), 3);
    let (kept, tally) = best(Some(steady.clone()), noisy.clone(), &rows, 210);
    assert_eq!(kept, steady);
    assert_eq!(tally.errors(true), validate(&rows, 210, Some(&steady)));
    let (kept, _) = best(Some(noisy), steady.clone(), &rows, 210);
    assert_eq!(kept, steady);
}

#[test]
fn candidato_recusado_continua_como_ponto_de_partida() {
    let rows = flat();
    let model = Hybrid::new(shape(&plan()), 3);
    let errors = validate(&rows, 210, Some(&model));
    let Judged::Rejected(_, outcome, Some(kept)) =
        judge(&plan(), model.clone(), &rows, 210, None, errors)
    else {
        panic!("deveria ser recusado e guardado")
    };
    assert_eq!(outcome, Outcome::BehindStatistics);
    assert_eq!(kept, model);
}
