use super::start::command;
use crate::domain::app::Launch;

fn launch(arguments: &[&str], terminal: bool) -> Launch {
    Launch {
        arguments: arguments.iter().map(|word| (*word).to_owned()).collect(),
        terminal,
    }
}

#[test]
fn aplicativo_grafico_roda_direto() {
    assert_eq!(
        command(&launch(&["firefox", "--new-window"], false), "xterm"),
        ["firefox", "--new-window"]
    );
}

#[test]
fn aplicativo_de_terminal_abre_dentro_do_terminal_sem_shell() {
    assert_eq!(
        command(&launch(&["htop", "-d", "10"], true), "sparky-terminal"),
        ["sparky-terminal", "-e", "htop", "-d", "10"]
    );
}
