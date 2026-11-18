use crate::message::{Event, Reply, Request};
use crate::window::WindowInfo;

#[test]
fn a_lista_de_janelas_leva_app_pid_e_foco_sem_titulo() {
    let reply = Reply::Windows {
        windows: vec![WindowInfo {
            app_id: "com.obsproject.Studio".to_owned(),
            pid: Some(4242),
            workspace: Some(2),
            visible: true,
            focused: false,
            fullscreen: false,
        }],
    };
    let json = serde_json::to_string(&reply).unwrap_or_default();
    assert_eq!(
        json,
        r#"{"reply":"windows","windows":[{"app_id":"com.obsproject.Studio","pid":4242,"workspace":2,"visible":true,"focused":false,"fullscreen":false}]}"#
    );
    assert!(!json.contains("title"));
    assert_eq!(serde_json::from_str::<Reply>(&json).ok(), Some(reply));
}

#[test]
fn assinar_janelas_e_um_pedido_separado() {
    assert_eq!(
        serde_json::to_string(&Request::WatchWindows).unwrap_or_default(),
        r#"{"request":"watch_windows"}"#
    );
    assert_eq!(
        serde_json::to_string(&Event::WindowsChanged).unwrap_or_default(),
        r#""windows_changed""#
    );
}
