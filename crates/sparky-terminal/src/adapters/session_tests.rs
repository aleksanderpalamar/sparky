use std::time::{Duration, Instant};

use alacritty_terminal::term::Osc52;
use iced::futures::StreamExt;
use iced::futures::channel::mpsc;
use sparky_theme::{Rgb, TerminalPalette};

use crate::adapters::listener::{Listener, Signal};
use crate::adapters::session::{Session, Size};
use crate::domain::grid::Cell;
use crate::domain::launch::{Launch, Program};
use crate::domain::screen::Screen;

const PALETA: TerminalPalette = TerminalPalette::TOKYO_NIGHT;
const LIMITE: Duration = Duration::from_secs(10);

fn rodar(script: &str) -> Screen {
    let (sender, mut signals) = mpsc::unbounded();
    let launch = Launch {
        program: Program::Command {
            program: "sh".to_owned(),
            args: vec!["-c".to_owned(), script.to_owned()],
        },
        directory: None,
    };
    let size = Size {
        columns: 60,
        lines: 10,
        cell: Cell {
            width: 8.0,
            height: 16.0,
        },
        scale: 1.0,
    };
    let Ok(session) = Session::start(&launch, size, Listener(sender)) else {
        panic!("o PTY não abriu")
    };
    let inicio = Instant::now();
    while inicio.elapsed() < LIMITE {
        match iced::futures::executor::block_on(signals.next()) {
            Some(Signal::Exited) | None => break,
            Some(_) => {}
        }
    }
    session.screen(&PALETA)
}

fn texto(tela: &Screen) -> String {
    tela.rows
        .iter()
        .flatten()
        .map(|run| run.text.as_str())
        .collect()
}

#[test]
fn o_shell_escreve_na_grade_com_as_cores_do_tema() {
    let tela = rodar("printf 'antes \\033[31mvermelho\\033[0m depois'; sleep 0.2");
    assert!(
        texto(&tela).contains("antes vermelho depois"),
        "{}",
        texto(&tela)
    );
    let vermelho = tela
        .rows
        .iter()
        .flatten()
        .find(|run| run.text.contains("vermelho"));
    assert_eq!(vermelho.map(|run| run.style.fg), Some(PALETA.ansi[1]));
}

#[test]
fn o_shell_recebe_o_ambiente_de_cores_do_powerlevel10k() {
    let tela = rodar("printf '%s %s %s' \"$TERM\" \"$COLORTERM\" \"$TERM_PROGRAM\"; sleep 0.2");
    assert!(
        texto(&tela).contains("xterm-256color truecolor sparky-terminal"),
        "{}",
        texto(&tela)
    );
}

#[test]
fn a_pasta_inicial_e_respeitada() {
    let (sender, mut signals) = mpsc::unbounded();
    let launch = Launch {
        program: Program::Command {
            program: "sh".to_owned(),
            args: vec!["-c".to_owned(), "pwd; sleep 0.2".to_owned()],
        },
        directory: Some("/usr".into()),
    };
    let size = Size {
        columns: 40,
        lines: 4,
        cell: Cell {
            width: 8.0,
            height: 16.0,
        },
        scale: 1.0,
    };
    let Ok(session) = Session::start(&launch, size, Listener(sender)) else {
        panic!("o PTY não abriu")
    };
    while let Some(signal) = iced::futures::executor::block_on(signals.next()) {
        if matches!(signal, Signal::Exited) {
            break;
        }
    }
    assert!(texto(&session.screen(&PALETA)).contains("/usr"));
}

#[test]
fn o_terminal_so_aceita_copiar_por_osc_52_nunca_ler() {
    assert_eq!(
        crate::adapters::session::terminal_config().osc52,
        Osc52::OnlyCopy
    );
}

#[test]
fn acentos_combinados_ficam_junto_da_letra() {
    let tela = rodar("printf 'cafe\\314\\201'; sleep 0.2");
    assert!(texto(&tela).contains("cafe\u{301}"), "{:?}", texto(&tela));
}

#[test]
fn a_cor_do_cursor_trocada_por_osc_12_e_respeitada() {
    let tela = rodar("printf '\\033]12;#ff0000\\007'; sleep 0.2");
    assert_eq!(
        tela.cursor.map(|cursor| cursor.color),
        Some(Rgb::new(255, 0, 0))
    );
}

#[test]
fn em_hidpi_o_pty_recebe_o_tamanho_em_pixels_fisicos() {
    let tamanho = Size {
        columns: 80,
        lines: 24,
        cell: Cell {
            width: 8.0,
            height: 16.0,
        },
        scale: 2.0,
    };
    let janela = tamanho.window();
    assert_eq!(
        (
            janela.num_cols,
            janela.num_lines,
            janela.cell_width,
            janela.cell_height
        ),
        (80, 24, 16, 32)
    );
}
