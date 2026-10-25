use iced::Size;
use sparky_theme::{Palette, TerminalPalette};

use super::workspace::{TAB_HEIGHT, Workspace};
use crate::domain::grid::Cell;
use crate::domain::launch::{Launch, Program};

pub fn workspace() -> Workspace {
    Workspace::new(
        Launch {
            program: Program::Shell,
            directory: None,
        },
        Palette::TOKYO_NIGHT,
        TerminalPalette::TOKYO_NIGHT,
        Cell {
            width: 8.0,
            height: 16.0,
        },
    )
}

#[test]
fn cada_aba_preserva_titulo_e_fracao_de_rolagem() {
    let mut app = workspace();
    let Some(first) = app.tabs.active_id() else {
        panic!("sem aba")
    };
    let Some(tab) = app.tabs.active_mut() else {
        panic!("sem estado")
    };
    tab.value.title = "primeira".into();
    tab.value.wheel_rest = 0.7;
    let Some(second) = app.new_tab() else {
        panic!("sem nova aba")
    };
    app.select(first);
    assert_eq!(app.title(), "primeira");
    assert_eq!(app.tabs.active().map(|tab| tab.value.wheel_rest), Some(0.7));
    app.select(second);
    assert_eq!(app.title(), "Terminal");
}

#[test]
fn a_barra_de_abas_nao_ocupa_celulas_do_pty() {
    let mut app = workspace();
    app.new_tab();
    app.fit(Size::new(900.0, 560.0));
    for tab in app.tabs.iter() {
        assert_eq!(
            tab.value.size.lines,
            ((560.0 - TAB_HEIGHT - 36.0) / 16.0).floor() as usize
        );
        assert_eq!(tab.value.size.columns, 108);
    }
}

#[test]
fn novas_abas_abrem_shell_sem_repetir_o_comando_da_primeira() {
    let mut app = workspace();
    app.launch.directory = Some("/tmp".into());
    app.launch.program = Program::Command {
        program: "htop".into(),
        args: Vec::new(),
    };
    app.new_tab();
    let Some(tab) = app.tabs.active() else {
        panic!("sem aba")
    };
    assert_eq!(tab.value.launch.program, Program::Shell);
    assert_eq!(tab.value.launch.directory, Some("/tmp".into()));
}

#[test]
fn fechar_a_ultima_aba_pede_encerramento_e_eventos_repetidos_sao_inofensivos() {
    let mut app = workspace();
    let Some(first) = app.tabs.active_id() else {
        panic!("sem aba")
    };
    let Some(second) = app.new_tab() else {
        panic!("sem segunda")
    };
    assert!(!app.close(first));
    assert_eq!(app.tabs.active_id(), Some(second));
    assert!(!app.close(first));
    assert!(app.close(second));
}

#[test]
fn mudar_escala_e_tema_atualiza_todas_as_abas() {
    let mut app = workspace();
    app.new_tab();
    app.rescale(1.5);
    let colors = TerminalPalette::from_colors("background = \"#112233\"");
    app.observe_colors(Palette::TOKYO_NIGHT, colors);
    for tab in app.tabs.iter() {
        assert_eq!(tab.value.size.scale, 1.5);
        assert_eq!(tab.value.palette, colors);
    }
}
