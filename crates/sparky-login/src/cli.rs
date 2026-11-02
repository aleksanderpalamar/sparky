pub const WORKER: &str = "--worker";
pub const DEFAULT_VT: u32 = 1;
pub const USAGE: &str = "uso: sparky-login [--vt <número>]";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Daemon { vt: u32 },
    Worker,
}

pub fn parse(args: impl IntoIterator<Item = String>) -> Option<Command> {
    let args: Vec<String> = args.into_iter().collect();
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] => Some(Command::Daemon { vt: DEFAULT_VT }),
        [WORKER] => Some(Command::Worker),
        ["--vt", number] => number
            .parse()
            .ok()
            .filter(|vt| (1..=63).contains(vt))
            .map(|vt| Command::Daemon { vt }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, parse};

    fn ler(args: &[&str]) -> Option<Command> {
        parse(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn sem_argumentos_o_daemon_usa_o_terminal_1() {
        assert_eq!(ler(&[]), Some(Command::Daemon { vt: 1 }));
    }

    #[test]
    fn o_terminal_pode_ser_outro_para_testar() {
        assert_eq!(ler(&["--vt", "3"]), Some(Command::Daemon { vt: 3 }));
    }

    #[test]
    fn terminal_invalido_ou_argumento_estranho_e_erro() {
        for args in [
            &["--vt", "0"][..],
            &["--vt", "64"],
            &["--vt", "x"],
            &["--vt"],
            &["--outro"],
        ] {
            assert_eq!(ler(args), None, "{args:?}");
        }
    }

    #[test]
    fn o_worker_e_o_proprio_binario_com_a_chave() {
        assert_eq!(ler(&["--worker"]), Some(Command::Worker));
    }
}
