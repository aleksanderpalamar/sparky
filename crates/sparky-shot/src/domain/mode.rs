#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Region,
    Window,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invocation {
    Picker,
    Capture(Mode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    Unknown(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Unknown(argument) => write!(formatter, "uso: sparky-shot [region|window|output], não {argument:?}"),
        }
    }
}

pub fn parse(arguments: &[String]) -> Result<Invocation, CliError> {
    let Some(first) = arguments.first() else {
        return Ok(Invocation::Picker);
    };
    let mode = match first.as_str() {
        "region" => Mode::Region,
        "window" => Mode::Window,
        "output" => Mode::Output,
        other => return Err(CliError::Unknown(other.to_owned())),
    };
    Ok(Invocation::Capture(mode))
}

#[cfg(test)]
mod tests {
    use super::{CliError, Invocation, Mode, parse};

    fn ler(argumentos: &[&str]) -> Result<Invocation, CliError> {
        let argumentos: Vec<String> = argumentos.iter().map(|a| (*a).to_owned()).collect();
        parse(&argumentos)
    }

    #[test]
    fn sem_argumento_abre_o_seletor() {
        assert_eq!(ler(&[]), Ok(Invocation::Picker));
    }

    #[test]
    fn le_os_tres_modos() {
        assert_eq!(ler(&["region"]), Ok(Invocation::Capture(Mode::Region)));
        assert_eq!(ler(&["window"]), Ok(Invocation::Capture(Mode::Window)));
        assert_eq!(ler(&["output"]), Ok(Invocation::Capture(Mode::Output)));
    }

    #[test]
    fn modo_desconhecido_e_erro() {
        assert_eq!(ler(&["tudo"]), Err(CliError::Unknown("tudo".to_owned())));
    }
}
