use sparky_ipc::{GlassArea, GlassRect};

use super::glass::area;
use crate::domain::layout::{Geometry, Layout};

fn frame() -> Geometry {
    Layout::new(64).frame(4, false)
}

#[test]
fn visivel_o_vidro_cobre_so_o_fundo_do_dock_em_repouso() {
    let surface = frame();
    let expected = GlassRect {
        x: (surface.size.0 as i32 - 408) / 2,
        y: surface.size.1 as i32 - 17 - 100,
        width: 408,
        height: 100,
    };
    assert_eq!(
        area(&Layout::new(64), 4, surface, false, 12),
        GlassArea::Rects {
            rects: vec![expected],
            radius: 12
        }
    );
}

#[test]
fn oculto_nao_ha_vidro() {
    assert_eq!(
        area(&Layout::new(64), 4, frame(), true, 12),
        GlassArea::Rects {
            rects: Vec::new(),
            radius: 12
        }
    );
}

#[test]
fn o_dock_anuncia_o_fundo_ao_abrir_e_area_vazia_ao_se_esconder_sem_repetir() {
    use super::message::Message;
    use super::state::Dock;
    use super::test_support::{Board, Memory, Started};
    use super::update::update;
    use crate::domain::presence::Seen;
    use crate::domain::support::app;

    let board = Board::default();
    let mut dock = Dock::new(
        sparky_theme::Skin::default(),
        Box::new(Memory::default()),
        Box::new(Started::default()),
        Box::new(board.clone()),
    );
    let _ = update(
        &mut dock,
        Message::Catalog(vec![app("sparky-files", "Arquivos")]),
    );
    let _ = update(
        &mut dock,
        Message::Catalog(vec![app("sparky-files", "Arquivos")]),
    );
    let vazio =
        |area: &GlassArea| matches!(area, GlassArea::Rects { rects, .. } if rects.is_empty());
    assert_eq!(board.0.borrow().len(), 1);
    assert!(!vazio(&board.0.borrow()[0]));
    let oculto = sparky_config::Dock {
        autohide: true,
        ..sparky_config::Dock::default()
    };
    let _ = update(&mut dock, Message::Preferences(oculto));
    let janela = Seen {
        app_id: "firefox".to_owned(),
        focused: false,
        visible: true,
    };
    let _ = update(&mut dock, Message::Windows(vec![janela]));
    let inicio = std::time::Instant::now();
    for quadro in 0..120 {
        let agora = inicio + std::time::Duration::from_millis(16 * quadro);
        let _ = update(&mut dock, Message::Frame(agora));
    }
    assert!(board.0.borrow().last().is_some_and(vazio));
}
