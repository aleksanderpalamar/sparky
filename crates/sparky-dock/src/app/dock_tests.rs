use super::message::Message;
use super::test_support::{Memory, Started, dock};
use super::update::update;
use crate::domain::layout::{Geometry, Layout, PEEK};
use crate::domain::presence::Seen;
use crate::domain::region::Stage;
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

fn settle(dock: &mut super::state::Dock) {
    let start = std::time::Instant::now();
    for frame in 0..120 {
        let now = start + std::time::Duration::from_millis(16 * frame);
        let _ = update(dock, Message::Frame(now));
    }
    assert!(!super::animate::animating(dock));
}

fn ready(memory: &Memory) -> super::state::Dock {
    let mut dock = dock(memory, &Started::default(), Vec::new());
    let _ = update(&mut dock, Message::Catalog(catalog()));
    dock
}

fn frame(apps: usize) -> Geometry {
    Layout::new(64).frame(apps, false)
}

fn stage(dock: &super::state::Dock) -> Stage {
    dock.stage()
}

#[test]
fn a_superficie_so_existe_com_o_catalogo_lido_e_o_dock_ligado() {
    let mut dock = dock(&Memory::default(), &Started::default(), Vec::new());
    assert_eq!(dock.desired(), None);
    let _ = update(&mut dock, Message::Catalog(catalog()));
    assert_eq!(dock.surface, Some(frame(2)));
    assert_eq!(
        dock.accepting.map(|rect| (rect.width, rect.height)),
        Some((258, 117))
    );
    let desligado = sparky_config::Dock {
        enabled: false,
        ..sparky_config::Dock::default()
    };
    let _ = update(&mut dock, Message::Preferences(desligado));
    assert_eq!((dock.surface, dock.accepting), (None, None));
    let _ = update(
        &mut dock,
        Message::Preferences(sparky_config::Dock::default()),
    );
    assert!(dock.surface.is_some() && dock.accepting.is_some());
}

#[test]
fn mudar_o_tamanho_redimensiona_na_hora() {
    let mut dock = ready(&Memory::default());
    let pequeno = sparky_config::Dock {
        size: 40,
        ..sparky_config::Dock::default()
    };
    let _ = update(&mut dock, Message::Preferences(pequeno));
    assert_eq!(dock.surface, Some(Layout::new(40).frame(2, false)));
    assert!(dock.accepting.is_some_and(|rect| rect.height < 117));
}

#[test]
fn passar_o_mouse_ou_abrir_o_menu_nao_redimensiona_a_superficie() {
    let mut dock = ready(&Memory::default());
    let quadro = dock.surface;
    let _ = update(&mut dock, Message::Hover(true));
    assert_eq!((dock.surface, stage(&dock)), (quadro, Stage::Lifted));
    let _ = update(&mut dock, Message::Menu("sparky-files".to_owned()));
    assert_eq!((dock.surface, stage(&dock)), (quadro, Stage::Menu));
    let _ = update(&mut dock, Message::Hover(false));
    assert_eq!(dock.menu, None);
    assert_eq!((dock.surface, stage(&dock)), (quadro, Stage::Resting));
}

#[test]
fn ocultar_automaticamente_desliza_e_so_a_faixa_recebe_o_mouse() {
    let mut dock = ready(&Memory::default());
    let oculto = sparky_config::Dock {
        autohide: true,
        ..sparky_config::Dock::default()
    };
    let _ = update(&mut dock, Message::Preferences(oculto));
    let quadro = dock.surface;
    assert_eq!(quadro.map(|geometry| geometry.zone), Some(0));
    let _ = update(&mut dock, Message::Windows(vec![window(true)]));
    assert!(super::animate::animating(&dock));
    assert_eq!(stage(&dock), Stage::Resting);
    settle(&mut dock);
    assert_eq!(stage(&dock), Stage::Tucked);
    assert_eq!(dock.accepting.map(|rect| rect.height), Some(PEEK as i32));
    let _ = update(&mut dock, Message::Hover(true));
    assert_eq!(stage(&dock), Stage::Lifted);
    settle(&mut dock);
    assert_eq!(dock.slide.value, 0.0);
    let _ = update(&mut dock, Message::Hover(false));
    settle(&mut dock);
    assert_eq!(stage(&dock), Stage::Tucked);
    let _ = update(&mut dock, Message::Windows(Vec::new()));
    settle(&mut dock);
    assert_eq!(stage(&dock), Stage::Resting);
    assert_eq!(dock.surface, quadro);
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
    assert_eq!(dock.surface, Some(frame(1)));
    assert_eq!(dock.accepting.map(|rect| rect.width), Some(183));
}

#[test]
fn dock_esvaziado_continua_vazio_na_proxima_sessao() {
    let memory = Memory::default();
    *memory.stored.borrow_mut() = Some(Vec::new());
    let dock = dock(&memory, &Started::default(), catalog());
    assert!(dock.pinned.ids().is_empty());
}
