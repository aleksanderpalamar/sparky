use std::time::{Duration, Instant};

use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::Term;
use iced::futures::channel::mpsc;
use sparky_theme::TerminalPalette;

use crate::adapters::greeting::{feed, run};
use crate::adapters::listener::Listener;
use crate::adapters::session::{Size, terminal_config};
use crate::adapters::snapshot::snapshot;
use crate::domain::grid::Cell;

const PRAZO: Duration = Duration::from_millis(300);

fn argumentos(lista: &[&str]) -> Vec<String> {
    lista.iter().map(|item| (*item).to_owned()).collect()
}

#[test]
fn a_saida_do_programa_volta_quando_ele_termina_bem() {
    let saida = run("printf", &argumentos(&["ol\\303\\241\\n"]), PRAZO);
    assert_eq!(saida.as_deref(), Some("olá\n".as_bytes()));
}

#[test]
fn falha_ou_saida_vazia_nao_viram_saudacao() {
    assert_eq!(
        run("sh", &argumentos(&["-c", "echo oi; exit 3"]), PRAZO),
        None
    );
    assert_eq!(run("true", &[], PRAZO), None);
    assert_eq!(run("programa-que-nao-existe-no-sparky", &[], PRAZO), None);
}

#[test]
fn programa_lento_e_encerrado_no_prazo() {
    let inicio = Instant::now();
    assert_eq!(run("sleep", &argumentos(&["5"]), PRAZO), None);
    assert!(inicio.elapsed() < Duration::from_secs(2));
}

#[test]
fn a_saudacao_vai_para_a_tela_antes_do_shell() {
    let (sender, _signals) = mpsc::unbounded();
    let tamanho = Size {
        columns: 40,
        lines: 6,
        cell: Cell {
            width: 8.0,
            height: 16.0,
        },
        scale: 1.0,
    };
    let term = FairMutex::new(Term::new(terminal_config(), &tamanho, Listener(sender)));
    feed(&term, b"Sparky\nKernel: Linux\n");
    let tela = snapshot(&term.lock(), &TerminalPalette::TOKYO_NIGHT);
    let linhas: Vec<String> = tela
        .rows
        .iter()
        .map(|row| row.iter().map(|run| run.text.as_str()).collect::<String>())
        .map(|linha| linha.trim_end().to_owned())
        .collect();
    assert_eq!(&linhas[..3], ["Sparky", "Kernel: Linux", ""]);
}

#[test]
fn filho_que_segura_a_saida_nao_passa_do_prazo() {
    let inicio = Instant::now();
    let saida = run("sh", &argumentos(&["-c", "sleep 5 & echo oi"]), PRAZO);
    assert!(inicio.elapsed() < Duration::from_secs(2), "{saida:?}");
}
