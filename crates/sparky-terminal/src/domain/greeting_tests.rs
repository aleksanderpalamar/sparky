use std::cell::Cell;

use sparky_config::Greeting;

use crate::domain::greeting::{arguments, screen_text, wanted};
use crate::domain::launch::{Launch, Program};

fn shell() -> Launch {
    Launch {
        program: Program::Shell,
        directory: None,
    }
}

fn comando() -> Launch {
    Launch {
        program: Program::Command {
            program: "htop".to_owned(),
            args: Vec::new(),
        },
        directory: None,
    }
}

#[test]
fn so_o_shell_interativo_com_a_preferencia_ligada_recebe_o_fetch() {
    assert_eq!(wanted(&shell(), || Greeting::Fetch), Greeting::Fetch);
    assert_eq!(wanted(&shell(), || Greeting::Quiet), Greeting::Quiet);
    assert_eq!(wanted(&comando(), || Greeting::Fetch), Greeting::Quiet);
}

#[test]
fn com_comando_a_preferencia_nem_e_lida() {
    let lida = Cell::new(false);
    let _ = wanted(&comando(), || {
        lida.set(true);
        Greeting::Fetch
    });
    assert!(!lida.get());
}

#[test]
fn o_fetch_e_pedido_visual_colorido_na_largura_da_aba() {
    assert_eq!(
        arguments(92),
        ["--format", "visual", "--color", "always", "--width", "92"]
    );
    assert_eq!(arguments(0).last().map(String::as_str), Some("1"));
}

#[test]
fn quebras_de_linha_viram_retorno_e_avanco_na_tela() {
    assert_eq!(screen_text(b"a\nb\r\nc\n"), b"a\r\nb\r\nc\r\n");
    assert_eq!(screen_text(b""), b"");
}
