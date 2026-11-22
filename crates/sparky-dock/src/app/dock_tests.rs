use super::message::Message;
use super::test_support::{Memory, Started, dock};
use super::update::update;
use crate::domain::layout::{MENU_SPACE, PEEK};
use crate::domain::presence::Seen;
use crate::domain::support::{app, ids};

fn catalog() -> Vec<crate::domain::app::App> {
    vec![
        app("sparky-files", "Arquivos"),
        app("sparky-store", "Loja"),
        app("firefox", "Firefox"),
    ]
}

fn window(visible: bool) -> Seen {
    Seen {
        app_id: "firefox".to_owned(),
        focused: false,
        visible,
    }
}

fn ready(memory: &Memory) -> super::state::Dock {
    let mut dock = dock(memory, &Started::default(), Vec::new());
    let _ = update(&mut dock, Message::Catalog(catalog()));
    dock
}

#[test]
fn a_superficie_so_existe_com_o_catalogo_lido_e_o_dock_ligado() {
    let mut dock = dock(&Memory::default(), &Started::default(), Vec::new());
    assert_eq!(dock.desired(), None);
    let _ = update(&mut dock, Message::Catalog(catalog()));
    assert_eq!(dock.surface.map(|geometry| geometry.size), Some((258, 117)));
    let desligado = sparky_config::Dock {
        enabled: false,
        ..sparky_config::Dock::default()
    };
    let _ = update(&mut dock, Message::Preferences(desligado));
    assert_eq!(dock.surface, None);
    let _ = update(
        &mut dock,
        Message::Preferences(sparky_config::Dock::default()),
    );
    assert!(dock.surface.is_some());
}

#[test]
fn mudar_o_tamanho_redimensiona_na_hora() {
    let mut dock = ready(&Memory::default());
    let pequeno = sparky_config::Dock {
        size: 40,
        ..sparky_config::Dock::default()
    };
    let _ = update(&mut dock, Message::Preferences(pequeno));
    let Some(geometry) = dock.surface else {
        panic!("o dock deveria estar aberto");
    };
    assert!(geometry.size.1 < 117);
    assert_eq!(geometry.zone, geometry.size.1 as i32);
}

#[test]
fn ocultar_automaticamente_esconde_com_janela_e_volta_com_o_mouse() {
    let mut dock = ready(&Memory::default());
    let oculto = sparky_config::Dock {
        autohide: true,
        ..sparky_config::Dock::default()
    };
    let _ = update(&mut dock, Message::Preferences(oculto));
    assert_eq!(dock.surface.map(|geometry| geometry.zone), Some(0));
    let _ = update(&mut dock, Message::Windows(vec![window(true)]));
    assert_eq!(dock.surface.map(|geometry| geometry.size.1), Some(PEEK));
    let _ = update(&mut dock, Message::Hover(true));
    assert_eq!(dock.surface.map(|geometry| geometry.size.1), Some(117));
    let _ = update(&mut dock, Message::Hover(false));
    assert_eq!(dock.surface.map(|geometry| geometry.size.1), Some(PEEK));
    let _ = update(&mut dock, Message::Windows(Vec::new()));
    assert_eq!(dock.surface.map(|geometry| geometry.size.1), Some(117));
}

#[test]
fn o_menu_abre_acima_do_icone_e_fecha_quando_o_mouse_sai() {
    let mut dock = ready(&Memory::default());
    let _ = update(&mut dock, Message::Menu("sparky-files".to_owned()));
    assert_eq!(
        dock.surface.map(|geometry| geometry.size.1),
        Some(117 + MENU_SPACE)
    );
    let _ = update(&mut dock, Message::Hover(false));
    assert_eq!(dock.menu, None);
    assert_eq!(dock.surface.map(|geometry| geometry.size.1), Some(117));
}

#[test]
fn remover_do_dock_salva_e_encolhe() {
    let memory = Memory::default();
    let mut dock = ready(&memory);
    let _ = update(&mut dock, Message::Menu("sparky-files".to_owned()));
    let _ = update(&mut dock, Message::Unpin("sparky-files".to_owned()));
    assert_eq!(dock.menu, None);
    assert!(!dock.pinned.contains("sparky-files"));
    assert_eq!(
        memory.stored.borrow().clone(),
        Some(ids(&["sparky-settings", "sparky-terminal", "sparky-store"]))
    );
    assert_eq!(dock.surface.map(|geometry| geometry.size.0), Some(183));
}

#[test]
fn dock_esvaziado_continua_vazio_na_proxima_sessao() {
    let memory = Memory::default();
    *memory.stored.borrow_mut() = Some(Vec::new());
    let dock = dock(&memory, &Started::default(), catalog());
    assert!(dock.pinned.ids().is_empty());
}
