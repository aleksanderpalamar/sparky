use super::service::Service;
use crate::application::lifecycle::Fault;
use crate::domain::ServiceState;
use crate::protocol::{Reply, Request};

fn ready() -> Service {
    let mut service = Service::default();
    service.ready();
    service
}

#[test]
fn restauracao_confirmada_depois_sai_do_modo_seguro() {
    let mut service = ready();
    service.failed(Fault::RestoreUnconfirmed);
    assert_eq!(service.state(), ServiceState::Safe);
    service.restored();
    assert_ne!(service.state(), ServiceState::Safe);
}

#[test]
fn pedir_para_sair_do_modo_seguro_nao_libera_antes_de_confirmar() {
    let mut service = ready();
    service.failed(Fault::RestoreUnconfirmed);
    assert_eq!(service.answer(Request::ClearSafeMode), Reply::Done);
    assert_eq!(service.state(), ServiceState::Safe);
}
