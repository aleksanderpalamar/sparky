use super::app::App;
use super::presence::{Presence, Seen, presence};
use super::support::app;

fn seen(app_id: &str, focused: bool) -> Seen {
    Seen {
        app_id: app_id.to_owned(),
        focused,
    }
}

#[test]
fn sem_janela_o_aplicativo_esta_fechado() {
    assert_eq!(
        presence(&app("firefox", "Firefox"), &[seen("steam", true)]),
        Presence::Closed
    );
}

#[test]
fn janela_com_o_mesmo_id_indica_execucao_e_foco() {
    let firefox = app("firefox", "Firefox");
    assert_eq!(
        presence(&firefox, &[seen("Firefox", false)]),
        Presence::Running
    );
    assert_eq!(
        presence(&firefox, &[seen("firefox", false), seen("firefox", true)]),
        Presence::Focused
    );
}

#[test]
fn reconhece_a_classe_declarada_o_programa_e_o_id_reverso() {
    let code = App {
        window_class: Some("Code".to_owned()),
        ..app("visual-studio-code", "Code")
    };
    assert_eq!(presence(&code, &[seen("code", false)]), Presence::Running);
    let mut nautilus = app("org.gnome.Nautilus", "Arquivos");
    nautilus.launch.arguments = vec!["/usr/bin/nautilus".to_owned()];
    assert_eq!(
        presence(&nautilus, &[seen("nautilus", false)]),
        Presence::Running
    );
    assert_eq!(
        presence(
            &app("com.valvesoftware.Steam", "Steam"),
            &[seen("steam", false)]
        ),
        Presence::Running
    );
}
