use std::time::{Duration, Instant};

use iced::futures::StreamExt;
use iced::futures::channel::mpsc;
use sparky_theme::TerminalPalette;

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
