use super::message::{Envelope, Refusal, Reply, Request, Status};
use crate::application::Progress;
use crate::domain::{Activities, Activity, Mode, ServiceState, Switch, TrainingState};

fn json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

#[test]
fn o_status_explica_estado_modo_e_atividades() {
    let status = Status {
        service: "0.32.0".to_owned(),
        state: ServiceState::Observing,
        enabled: Switch::On,
        mode: Mode::Observation,
        learning: Switch::On,
        training: TrainingState::Allowed,
        activities: Activities::default().with(Activity::Gaming),
        progress: Progress::default(),
    };
    let text = json(&Envelope::current(Reply::Status(status.clone())));
    assert_eq!(
        text,
        r#"{"version":1,"reply":"status","service":"0.32.0","state":"observing","enabled":"on","mode":"observation","learning":"on","training":{"training":"allowed"},"activities":["gaming"],"progress":{"collected":0,"needed":0,"epoch":0,"epochs":0,"rounds":0,"last":null}}"#
    );
    let back: Option<Envelope<Reply>> = serde_json::from_str(&text).ok();
    assert_eq!(
        back.map(|envelope| envelope.body),
        Some(Reply::Status(status))
    );
}

#[test]
fn a_recusa_diz_o_motivo() {
    assert_eq!(
        json(&Reply::Refused {
            reason: Refusal::TooLarge
        }),
        r#"{"reply":"refused","reason":"too_large"}"#
    );
}

#[test]
fn os_pedidos_de_configuracao_sao_tipados() {
    assert_eq!(
        json(&Request::SetEnabled {
            switch: Switch::Off
        }),
        r#"{"request":"set_enabled","switch":"off"}"#
    );
    assert_eq!(
        json(&Request::SetLearning { switch: Switch::On }),
        r#"{"request":"set_learning","switch":"on"}"#
    );
    assert_eq!(
        json(&Request::ClearSafeMode),
        r#"{"request":"clear_safe_mode"}"#
    );
}

#[test]
fn status_sem_progresso_de_versao_anterior_continua_valido() {
    let text = r#"{"version":1,"reply":"status","service":"0.32.0","state":"observing","enabled":"on","mode":"observation","learning":"on","training":{"training":"allowed"},"activities":[]}"#;
    let back: Option<Envelope<Reply>> = serde_json::from_str(text).ok();
    let Some(Reply::Status(status)) = back.map(|envelope| envelope.body) else {
        panic!("status antigo deveria ser aceito")
    };
    assert_eq!(status.progress, Progress::default());
}
