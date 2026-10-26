#[path = "scroll_support.rs"]
mod support;

use iced::{Point, Size, mouse};

use super::message::Message;
use super::update::update;
use support::{populated, scripted, text, wait_for, wheel};

#[test]
fn roda_do_mouse_mostra_saida_antiga_e_volta_ao_fim() {
    let mut app = populated();
    let current = text(&app);
    assert!(current.contains("linha-119"));
    let at = Point::new(200.0, 150.0);
    let messages = wheel(&app, mouse::ScrollDelta::Lines { x: 0.0, y: 10.0 }, at);
    assert!(
        matches!(messages.as_slice(), [Message::Wheel(_, 10.0, _)]),
        "{messages:?}"
    );
    for message in messages {
        let _ = update(&mut app, message);
    }
    assert_ne!(text(&app), current);
    assert!(!text(&app).contains("PRONTO"));
    for message in wheel(&app, mouse::ScrollDelta::Lines { x: 0.0, y: -10.0 }, at) {
        let _ = update(&mut app, message);
    }
    assert_eq!(text(&app), current);
}

#[test]
fn programa_que_pede_mouse_recebe_roda_em_vez_de_perder_evento() {
    for modes in ["?1000h", "?1002h", "?1003h", "?1049h\\033[?1000h"] {
        let script = format!(
            "stty -echo -icanon min 1 time 0; printf '\\033[{modes}\\033[?1006hPRONTO'; dd bs=1 count=10 2>/dev/null | od -An -tx1; printf 'RECEBIDO'; read -r fim"
        );
        let mut app = scripted(&script);
        let flags = app
            .tabs
            .active()
            .unwrap()
            .value
            .session
            .as_ref()
            .unwrap()
            .mode();
        assert!(flags.intersects(alacritty_terminal::term::TermMode::MOUSE_MODE));
        let at = Point::new(54.0, super::workspace::TAB_HEIGHT + 58.0);
        for message in wheel(&app, mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 }, at) {
            let _ = update(&mut app, message);
        }
        wait_for(&mut app, "RECEBIDO");
        assert!(
            text(&app).contains("1b 5b 3c 36 34 3b 35 3b 33 4d"),
            "{}",
            text(&app)
        );
    }
}

#[test]
fn rolagem_em_pixels_acumula_fracoes_sem_afetar_a_barra_de_abas() {
    let mut app = populated();
    let at = Point::new(200.0, 150.0);
    let current = text(&app);
    let delta = mouse::ScrollDelta::Pixels { x: 0.0, y: 8.0 };
    for message in wheel(&app, delta, Point::new(50.0, 10.0)) {
        let _ = update(&mut app, message);
    }
    assert_eq!(text(&app), current);
    for message in wheel(&app, delta, at) {
        let _ = update(&mut app, message);
    }
    assert_eq!(text(&app), current);
    for message in wheel(&app, delta, at) {
        let _ = update(&mut app, message);
    }
    assert_ne!(text(&app), current);
}

#[test]
fn formato_pedido_pelo_programa_define_a_codificacao_da_roda() {
    for (modes, count, expected) in [
        ("", 6, "1b 5b 4d 60 7f 23"),
        ("\\033[?1005h", 7, "1b 5b 4d 60 c2 9e 23"),
        (
            "\\033[?1005h\\033[?1006h",
            12,
            "1b 5b 3c 36 34 3b 31 32 36 3b 33 4d",
        ),
    ] {
        let script = format!(
            "stty -echo -icanon min 1 time 0; printf '\\033[?1000h{modes}PRONTO'; dd bs=1 count={count} 2>/dev/null | od -An -tx1; printf 'RECEBIDO'; read -r fim"
        );
        let mut app = scripted(&script);
        app.fit(Size::new(1200.0, 256.0));
        let x = match modes.is_empty() {
            true => 774.0,
            false => 1022.0,
        };
        let at = Point::new(x, super::workspace::TAB_HEIGHT + 58.0);
        for message in wheel(&app, mouse::ScrollDelta::Lines { x: 0.0, y: 1.0 }, at) {
            let _ = update(&mut app, message);
        }
        wait_for(&mut app, "RECEBIDO");
        assert!(text(&app).contains(expected), "{}", text(&app));
    }
}
