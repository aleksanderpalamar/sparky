use crate::message::{Event, Reply, Request, WorkspaceInfo};

#[test]
fn o_pedido_vira_json_com_o_nome_em_minusculas() {
    let json = serde_json::to_string(&Request::FocusWorkspace { id: 3 }).unwrap_or_default();
    assert_eq!(json, r#"{"request":"focus_workspace","id":3}"#);
}

#[test]
fn a_captura_leva_a_area_e_o_caminho() {
    let request = crate::message::Request::Capture {
        area: crate::message::CaptureArea::Region {
            output: None,
            x: 1,
            y: 2,
            width: 3,
            height: 4,
        },
        path: "/tmp/a.png".to_owned(),
    };
    let json = serde_json::to_string(&request).unwrap_or_default();
    assert_eq!(
        json,
        r#"{"request":"capture","area":{"kind":"region","output":null,"x":1,"y":2,"width":3,"height":4},"path":"/tmp/a.png"}"#
    );
    assert_eq!(
        serde_json::from_str::<crate::message::Request>(&json).ok(),
        Some(request)
    );
}

#[test]
fn recarregar_a_configuracao_e_um_pedido_sem_campos() {
    let json = serde_json::to_string(&Request::ReloadConfig).unwrap_or_default();
    assert_eq!(json, r#"{"request":"reload_config"}"#);
}

#[test]
fn o_evento_vai_dentro_da_resposta() {
    let json = serde_json::to_string(&Reply::Event {
        event: Event::WorkspacesChanged,
    })
    .unwrap_or_default();
    assert_eq!(json, r#"{"reply":"event","event":"workspaces_changed"}"#);
}

#[test]
fn a_lista_de_monitores_diz_qual_esta_em_foco() {
    let reply = Reply::Outputs {
        outputs: vec![crate::output::OutputInfo {
            name: "DP-1".to_owned(),
            x: 1920,
            y: 0,
            width: 2560,
            height: 1440,
            focused: true,
            ..crate::output::OutputInfo::default()
        }],
    };
    let json = serde_json::to_string(&reply).unwrap_or_default();
    let inicio = r#"{"reply":"outputs","outputs":[{"name":"DP-1","x":1920,"y":0,"width":2560,"height":1440,"focused":true,"#;
    assert!(json.starts_with(inicio), "{json}");
    assert_eq!(serde_json::from_str::<Reply>(&json).ok(), Some(reply));
}

#[test]
fn a_lista_de_areas_volta_inteira() {
    let reply = Reply::Workspaces {
        workspaces: vec![WorkspaceInfo {
            id: 2,
            monitor: "HDMI-A-1".to_owned(),
            windows: 1,
            visible: true,
            focused: true,
        }],
    };
    let json = serde_json::to_string(&reply).unwrap_or_default();
    assert_eq!(serde_json::from_str::<Reply>(&json).ok(), Some(reply));
}

#[test]
fn o_pedido_de_sessao_leva_a_acao_pelo_nome() {
    use crate::message::{Request, SessionAction};
    let pedido = Request::Session {
        action: SessionAction::Poweroff,
    };
    let json = serde_json::to_string(&pedido).unwrap_or_default();
    assert_eq!(json, r#"{"request":"session","action":"poweroff"}"#);
    assert_eq!(serde_json::from_str::<Request>(&json).ok(), Some(pedido));
    assert_eq!(
        SessionAction::named("suspend"),
        Some(SessionAction::Suspend)
    );
    assert_eq!(SessionAction::named("voar"), None);
    assert_eq!(SessionAction::named("unlock"), Some(SessionAction::Unlock));
}

#[test]
fn o_pedido_de_inatividade_diz_se_ha_quem_segure() {
    use crate::message::IdleHold;
    let json = |hold| serde_json::to_string(&Request::Idle { hold }).unwrap_or_default();
    assert_eq!(json(IdleHold::Held), r#"{"request":"idle","hold":"held"}"#);
    assert_eq!(json(IdleHold::Free), r#"{"request":"idle","hold":"free"}"#);
}

#[test]
fn bloquear_e_desbloquear_viram_eventos() {
    let json = |event| serde_json::to_string(&Reply::Event { event }).unwrap_or_default();
    assert_eq!(json(Event::Locked), r#"{"reply":"event","event":"locked"}"#);
    assert_eq!(
        json(Event::Unlocked),
        r#"{"reply":"event","event":"unlocked"}"#
    );
}

#[test]
fn o_pedido_de_informacoes_da_sessao_e_um_pedido_sem_campos() {
    let json = serde_json::to_string(&Request::Info).unwrap_or_default();
    assert_eq!(json, r#"{"request":"info"}"#);
    assert_eq!(
        serde_json::from_str::<Request>(&json).ok(),
        Some(Request::Info)
    );
}

#[test]
fn a_resposta_de_informacoes_volta_inteira() {
    use crate::info::{SessionInfo, SessionKind};
    let reply = Reply::Info {
        info: SessionInfo {
            version: "0.30.0".to_owned(),
            display: "wayland-1".to_owned(),
            session: SessionKind::Real,
            theme: Some("tokyo-night".to_owned()),
            accent: "#7aa2f7".to_owned(),
            swatches: vec!["#1a1b26".to_owned(), "#f7768e".to_owned()],
        },
    };
    let json = serde_json::to_string(&reply).unwrap_or_default();
    assert!(
        json.starts_with(
            r#"{"reply":"info","info":{"version":"0.30.0","display":"wayland-1","session":"real""#
        ),
        "{json}"
    );
    assert_eq!(serde_json::from_str::<Reply>(&json).ok(), Some(reply));
}

#[test]
fn informacoes_sem_os_campos_novos_ainda_sao_lidas() {
    use crate::info::{SessionInfo, SessionKind};
    let json = r#"{"reply":"info","info":{"version":"0.30.0","display":"wayland-1"}}"#;
    let Ok(Reply::Info { info }) = serde_json::from_str::<Reply>(json) else {
        panic!("deveria ler a resposta antiga");
    };
    assert_eq!(
        info,
        SessionInfo {
            version: "0.30.0".to_owned(),
            display: "wayland-1".to_owned(),
            session: SessionKind::Unknown,
            theme: None,
            accent: String::new(),
            swatches: Vec::new(),
        }
    );
}
