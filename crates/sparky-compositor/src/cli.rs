#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Options {
    pub spawn: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    MissingCommand,
    Unknown(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::MissingCommand => write!(formatter, "--spawn precisa de um comando"),
            CliError::Unknown(argument) => write!(formatter, "argumento desconhecido: {argument}"),
        }
    }
}

impl std::error::Error for CliError {}

impl Options {
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, CliError> {
        let mut options = Self::default();
        let mut arguments = arguments.into_iter();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--spawn" => options.spawn.push(arguments.next().ok_or(CliError::MissingCommand)?),
                _ => return Err(CliError::Unknown(argument)),
            }
        }
        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::{CliError, Options};

    fn ler(argumentos: &[&str]) -> Result<Options, CliError> {
        Options::parse(argumentos.iter().map(|argumento| (*argumento).to_owned()))
    }

    #[test]
    fn sem_argumentos_nao_abre_nada() {
        assert_eq!(ler(&[]), Ok(Options::default()));
    }

    #[test]
    fn spawn_pode_se_repetir() {
        let options = ler(&["--spawn", "kitty", "--spawn", "fuzzel"]);
        assert_eq!(options.map(|o| o.spawn), Ok(vec!["kitty".to_owned(), "fuzzel".to_owned()]));
    }

    #[test]
    fn spawn_sem_comando_e_erro() {
        assert_eq!(ler(&["--spawn"]), Err(CliError::MissingCommand));
    }

    #[test]
    fn argumento_desconhecido_e_erro() {
        assert_eq!(ler(&["--hyprland"]), Err(CliError::Unknown("--hyprland".to_owned())));
    }
}
