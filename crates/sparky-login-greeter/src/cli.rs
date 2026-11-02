use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Preview(Options),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    pub wallpaper: Option<PathBuf>,
}

pub const USAGE: &str = "uso: sparky-login-greeter --preview [--wallpaper <imagem>]";

pub fn parse(args: impl IntoIterator<Item = String>) -> Option<Command> {
    let mut preview = false;
    let mut options = Options::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--preview" => preview = true,
            "--wallpaper" => options.wallpaper = Some(PathBuf::from(args.next()?)),
            _ => return None,
        }
    }
    preview.then_some(Command::Preview(options))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Command, Options, parse};

    fn argumentos(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn a_previa_abre_com_ou_sem_papel_de_parede() {
        assert_eq!(
            parse(argumentos(&["--preview"])),
            Some(Command::Preview(Options::default()))
        );
        let escolhido = Options {
            wallpaper: Some(PathBuf::from("/tmp/fundo.png")),
        };
        assert_eq!(
            parse(argumentos(&["--preview", "--wallpaper", "/tmp/fundo.png"])),
            Some(Command::Preview(escolhido))
        );
    }

    #[test]
    fn sem_previa_argumento_desconhecido_ou_sem_valor_mostra_o_uso() {
        assert_eq!(parse(argumentos(&[])), None);
        assert_eq!(parse(argumentos(&["--preview", "--wallpaper"])), None);
        assert_eq!(parse(argumentos(&["--preview", "--outro"])), None);
    }
}
