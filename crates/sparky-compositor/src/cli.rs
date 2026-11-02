#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Desktop,
    LoginScreen {
        greeter: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Options {
    pub spawn: Vec<String>,
    pub mode: Mode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    MissingCommand,
    MissingGreeter,
    LoginScreenAlone,
    Unknown(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::MissingCommand => write!(formatter, "--spawn precisa de um comando"),
            CliError::MissingGreeter => write!(formatter, "--login-screen precisa de um comando"),
            CliError::LoginScreenAlone => {
                write!(formatter, "--login-screen não se combina com --spawn")
            }
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
                "--spawn" => options
                    .spawn
                    .push(arguments.next().ok_or(CliError::MissingCommand)?),
                "--login-screen" => {
                    let greeter = arguments.next().ok_or(CliError::MissingGreeter)?;
                    options.mode = Mode::LoginScreen { greeter };
                }
                _ => return Err(CliError::Unknown(argument)),
            }
        }
        match (&options.mode, options.spawn.is_empty()) {
            (Mode::LoginScreen { .. }, false) => Err(CliError::LoginScreenAlone),
            _ => Ok(options),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CliError, Mode, Options};

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
        assert_eq!(
            options.map(|o| o.spawn),
            Ok(vec!["kitty".to_owned(), "fuzzel".to_owned()])
        );
    }

    #[test]
    fn spawn_sem_comando_e_erro() {
        assert_eq!(ler(&["--spawn"]), Err(CliError::MissingCommand));
    }

    #[test]
    fn tela_de_login_abre_o_comando_no_modo_restrito() {
        let options = ler(&["--login-screen", "sparky-login-greeter"]);
        assert_eq!(
            options.map(|o| o.mode),
            Ok(Mode::LoginScreen {
                greeter: "sparky-login-greeter".to_owned()
            })
        );
    }

    #[test]
    fn tela_de_login_sem_comando_ou_com_spawn_e_erro() {
        assert_eq!(ler(&["--login-screen"]), Err(CliError::MissingGreeter));
        assert_eq!(
            ler(&["--login-screen", "tela", "--spawn", "kitty"]),
            Err(CliError::LoginScreenAlone)
        );
        assert_eq!(
            ler(&["--spawn", "kitty", "--login-screen", "tela"]),
            Err(CliError::LoginScreenAlone)
        );
    }

    #[test]
    fn sem_tela_de_login_o_modo_e_o_desktop() {
        assert_eq!(
            ler(&["--spawn", "kitty"]).map(|o| o.mode),
            Ok(Mode::Desktop)
        );
    }

    #[test]
    fn argumento_desconhecido_e_erro() {
        assert_eq!(
            ler(&["--hyprland"]),
            Err(CliError::Unknown("--hyprland".to_owned()))
        );
    }
}
