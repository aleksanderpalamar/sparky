use super::training::training;
use crate::domain::{Activities, Activity, ServiceState, Suspension, Switch, TrainingState};

fn doing(list: &[Activity]) -> Activities {
    list.iter().copied().collect()
}

#[test]
fn treina_quando_o_usuario_esta_em_atividade_calma() {
    let calm = doing(&[Activity::Desktop]);
    assert_eq!(
        training(Switch::On, ServiceState::Observing, calm),
        TrainingState::Allowed
    );
}

#[test]
fn jogo_com_obs_suspende_o_treinamento() {
    let session = doing(&[Activity::Gaming, Activity::Recording]);
    assert_eq!(
        training(Switch::On, ServiceState::Adaptive, session),
        TrainingState::Suspended(Suspension::SensitiveActivity)
    );
}

#[test]
fn pressao_critica_tem_motivo_proprio() {
    let pressure = doing(&[Activity::Gaming, Activity::CriticalPressure]);
    assert_eq!(
        training(Switch::On, ServiceState::Adaptive, pressure),
        TrainingState::Suspended(Suspension::Pressure)
    );
}

#[test]
fn a_escolha_do_usuario_e_o_orcamento_vem_antes_da_atividade() {
    let session = doing(&[Activity::Compiling]);
    assert_eq!(
        training(Switch::Off, ServiceState::Adaptive, session),
        TrainingState::Suspended(Suspension::UserChoice)
    );
    assert_eq!(
        training(Switch::On, ServiceState::Degraded, session),
        TrainingState::Suspended(Suspension::Budget)
    );
}

#[test]
fn sem_servico_ativo_o_treinamento_fica_indisponivel() {
    for state in [
        ServiceState::Disabled,
        ServiceState::Starting,
        ServiceState::Safe,
    ] {
        assert_eq!(
            training(Switch::On, state, Activities::default()),
            TrainingState::Unavailable,
            "{state:?}"
        );
    }
}
