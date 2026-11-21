use super::learning::{Learner, Plan, Verdict, shape};
use super::learning_support::{finish, fresh, sample, small, stubborn, wave};
use crate::application::encoding::Sample;
use crate::neural::Hybrid;

#[test]
fn a_rodada_deixa_o_diagnostico_com_os_mesmos_erros_do_veredito() {
    let mut learner = Learner::new(stubborn());
    wave(&mut learner, 400);
    let Verdict::Rejected(errors, _) = finish(&mut learner) else {
        panic!("deveria ser recusado")
    };
    let diagnosis = learner.diagnosis().map(|tally| tally.errors(true));
    assert_eq!(diagnosis, Some(errors));
    assert!(
        learner
            .diagnosis()
            .is_some_and(|tally| !tally.lines().is_empty())
    );
}

#[test]
fn retomar_restaura_coleta_e_candidato_compativel() {
    let mut learner = Learner::new(small());
    let samples: Vec<Sample> = (0..500).map(|index| sample(index as f64, 20.0)).collect();
    learner.resume(samples.clone(), Some(fresh(small())));
    assert_eq!(learner.progress().collected, 400);
    assert_eq!(learner.samples().next(), samples.get(100));
    assert!(learner.incumbent().is_some());
    let other = Hybrid::still(
        shape(&Plan {
            hidden: 4,
            ..small()
        }),
        1,
    );
    learner.resume(samples, Some(other));
    assert!(learner.incumbent().is_none());
}

#[test]
fn esquecer_limpa_coleta_candidato_e_diagnostico() {
    let mut learner = Learner::new(stubborn());
    wave(&mut learner, 400);
    finish(&mut learner);
    learner.forget();
    assert_eq!(learner.progress().collected, 0);
    assert!(learner.incumbent().is_none());
    assert!(learner.diagnosis().is_none());
}
