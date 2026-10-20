use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "request", rename_all = "snake_case")]
pub enum Request {
    Workspaces,
    FocusWorkspace { id: u32 },
    Subscribe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reply", rename_all = "snake_case")]
pub enum Reply {
    Done,
    Workspaces { workspaces: Vec<WorkspaceInfo> },
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

#[cfg(test)]
mod tests {
    use super::{Event, Reply, Request, WorkspaceInfo};

    #[test]
    fn o_pedido_vira_json_com_o_nome_em_minusculas() {
        let json = serde_json::to_string(&Request::FocusWorkspace { id: 3 }).unwrap_or_default();
        assert_eq!(json, r#"{"request":"focus_workspace","id":3}"#);
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
}
