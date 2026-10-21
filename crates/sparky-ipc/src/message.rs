use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "snake_case")]
pub enum Request {
    Workspaces,
    FocusWorkspace { id: u32 },
    Subscribe,
    ReloadConfig,
    Capture { area: CaptureArea, path: String },
    Outputs,
    Session { action: SessionAction },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionAction {
    Lock,
    Suspend,
    Logout,
    Reboot,
    Poweroff,
}

impl SessionAction {
    pub fn named(name: &str) -> Option<Self> {
        match name {
            "lock" => Some(SessionAction::Lock),
            "suspend" => Some(SessionAction::Suspend),
            "logout" => Some(SessionAction::Logout),
            "reboot" => Some(SessionAction::Reboot),
            "poweroff" => Some(SessionAction::Poweroff),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CaptureArea {
    Output {
        output: Option<String>,
    },
    Window,
    Region {
        output: Option<String>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "snake_case")]
pub enum Reply {
    Done,
    Workspaces { workspaces: Vec<WorkspaceInfo> },
    Outputs { outputs: Vec<OutputInfo> },
    Event { event: Event },
    Error { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    WorkspacesChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: u32,
    pub monitor: String,
    pub windows: usize,
    pub visible: bool,
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub focused: bool,
}

#[cfg(test)]
mod tests {
    use super::{Event, Reply, Request, WorkspaceInfo};

    #[test]
    fn o_pedido_vira_json_com_o_nome_em_minusculas() {
        let json = serde_json::to_string(&Request::FocusWorkspace { id: 3 }).unwrap_or_default();
        assert_eq!(json, r#"{"request":"focus_workspace","id":3}"#);
    }

    #[test]
    fn a_captura_leva_a_area_e_o_caminho() {
        let request = super::Request::Capture {
            area: super::CaptureArea::Region { output: None, x: 1, y: 2, width: 3, height: 4 },
            path: "/tmp/a.png".to_owned(),
        };
        let json = serde_json::to_string(&request).unwrap_or_default();
        assert_eq!(
            json,
            r#"{"request":"capture","area":{"kind":"region","output":null,"x":1,"y":2,"width":3,"height":4},"path":"/tmp/a.png"}"#
        );
        assert_eq!(serde_json::from_str::<super::Request>(&json).ok(), Some(request));
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
            outputs: vec![super::OutputInfo {
                name: "DP-1".to_owned(),
                x: 1920,
                y: 0,
                width: 2560,
                height: 1440,
                focused: true,
            }],
        };
        let json = serde_json::to_string(&reply).unwrap_or_default();
        assert_eq!(
            json,
            r#"{"reply":"outputs","outputs":[{"name":"DP-1","x":1920,"y":0,"width":2560,"height":1440,"focused":true}]}"#
        );
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
        use super::{Request, SessionAction};
        let pedido = Request::Session { action: SessionAction::Poweroff };
        let json = serde_json::to_string(&pedido).unwrap_or_default();
        assert_eq!(json, r#"{"request":"session","action":"poweroff"}"#);
        assert_eq!(serde_json::from_str::<Request>(&json).ok(), Some(pedido));
        assert_eq!(SessionAction::named("suspend"), Some(SessionAction::Suspend));
        assert_eq!(SessionAction::named("voar"), None);
    }
}
