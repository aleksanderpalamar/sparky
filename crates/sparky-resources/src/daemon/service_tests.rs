use super::service::Service;
use crate::domain::{Mode, ServiceState, Suspension, Switch, TrainingState};
use crate::protocol::{Refusal, Reply, Request, Status};

fn status(service: &mut Service) -> Option<Status> {
    match service.answer(Request::Status) {
        Reply::Status(status) => Some(status),
        _ => None,
    }
}

fn ready() -> Service {
    let mut service = Service::default();
    service.ready();
    service
}

#[test]
fn comeca_ligado_observando_e_podendo_aprender() {
    let status = status(&mut ready());
    assert_eq!(
        status.as_ref().map(|status| status.state),
        Some(ServiceState::Observing)
    );
    assert_eq!(
        status.as_ref().map(|status| status.enabled),
        Some(Switch::On)
    );
    assert_eq!(
        status.as_ref().map(|status| status.training),
        Some(TrainingState::Allowed)
    );
    assert_eq!(
        status.map(|status| status.service),
        Some(env!("CARGO_PKG_VERSION").to_owned())
    );
}

#[test]
fn desligar_volta_ao_comportamento_nativo() {
    let mut service = ready();
    assert_eq!(
        service.answer(Request::SetEnabled {
            switch: Switch::Off
        }),
        Reply::Done
    );
    let status = status(&mut service);
    assert_eq!(
        status.as_ref().map(|status| status.state),
        Some(ServiceState::Disabled)
    );
    assert_eq!(
        status.map(|status| status.training),
        Some(TrainingState::Unavailable)
    );
}

#[test]
fn o_modo_escolhido_aparece_no_status() {
    let mut service = ready();
    assert_eq!(
        service.answer(Request::SetMode {
            mode: Mode::Conservative
        }),
        Reply::Done
    );
    assert_eq!(
        status(&mut service).map(|status| status.mode),
        Some(Mode::Conservative)
    );
}

#[test]
fn desligar_o_aprendizado_suspende_o_treinamento() {
    let mut service = ready();
    assert_eq!(
        service.answer(Request::SetLearning {
            switch: Switch::Off
        }),
        Reply::Done
    );
    assert_eq!(
        status(&mut service).map(|status| status.training),
        Some(TrainingState::Suspended(Suspension::UserChoice))
    );
}

#[test]
fn so_libera_o_modo_seguro_quando_esta_nele() {
    let mut service = ready();
    assert_eq!(
        service.answer(Request::ClearSafeMode),
        Reply::Refused {
            reason: Refusal::NotInSafeMode
        }
    );
}
