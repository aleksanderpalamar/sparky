use std::time::Duration;

use super::cadence::interval;
use crate::domain::{Activities, Activity};

fn doing(list: &[Activity]) -> Activities {
    list.iter().copied().collect()
}

#[test]
fn ocioso_coleta_devagar() {
    assert_eq!(interval(Activities::default()), Duration::from_secs(5));
    assert_eq!(interval(doing(&[Activity::Idle])), Duration::from_secs(5));
}

#[test]
fn desktop_jogo_e_gravacao_coletam_a_cada_dois_segundos() {
    assert_eq!(
        interval(doing(&[Activity::Desktop])),
        Duration::from_secs(2)
    );
    assert_eq!(
        interval(doing(&[Activity::Gaming, Activity::Recording])),
        Duration::from_secs(2)
    );
}

#[test]
fn desenvolvimento_coleta_um_pouco_mais_rapido() {
    assert_eq!(
        interval(doing(&[Activity::Development])),
        Duration::from_millis(1500)
    );
    assert_eq!(
        interval(doing(&[Activity::Development, Activity::Compiling])),
        Duration::from_secs(2)
    );
}

#[test]
fn pressao_critica_acelera_temporariamente() {
    assert_eq!(
        interval(doing(&[Activity::Gaming, Activity::CriticalPressure])),
        Duration::from_millis(250)
    );
}
