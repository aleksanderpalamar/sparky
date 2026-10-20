use crate::message::Request;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    Missing,
    Unknown(String),
    InvalidWorkspace(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::Missing => write!(formatter, "uso: sparky-msg workspaces | focus-workspace <n> | subscribe"),
            CommandError::Unknown(name) => write!(formatter, "comando desconhecido: {name}"),
            CommandError::InvalidWorkspace(value) => write!(formatter, "área inválida: {value}"),
        }
    }
}

pub fn parse(arguments: &[String]) -> Result<Request, CommandError> {
    let Some(name) = arguments.first() else {
        return Err(CommandError::Missing);
    };
    match name.as_str() {
        "workspaces" => Ok(Request::Workspaces),
        "subscribe" => Ok(Request::Subscribe),
        "focus-workspace" => workspace(arguments.get(1)),
        other => Err(CommandError::Unknown(other.to_owned())),
    }
}

fn workspace(value: Option<&String>) -> Result<Request, CommandError> {
    let value = value.ok_or(CommandError::Missing)?;
    value
        .parse()
        .map(|id| Request::FocusWorkspace { id })
        .map_err(|_| CommandError::InvalidWorkspace(value.clone()))
}

#[cfg(test)]
mod tests {
    use super::{CommandError, parse};
    use crate::message::Request;

    fn ler(argumentos: &[&str]) -> Result<Request, CommandError> {
        let argumentos: Vec<String> = argumentos.iter().map(|a| (*a).to_owned()).collect();
        parse(&argumentos)
    }

    #[test]
    fn le_os_tres_comandos() {
        assert_eq!(ler(&["workspaces"]), Ok(Request::Workspaces));
        assert_eq!(ler(&["subscribe"]), Ok(Request::Subscribe));
        assert_eq!(ler(&["focus-workspace", "4"]), Ok(Request::FocusWorkspace { id: 4 }));
    }

    #[test]
    fn recusa_area_invalida_e_comando_desconhecido() {
        assert_eq!(ler(&["focus-workspace", "x"]), Err(CommandError::InvalidWorkspace("x".to_owned())));
        assert_eq!(ler(&["focus-workspace"]), Err(CommandError::Missing));
        assert_eq!(ler(&["voar"]), Err(CommandError::Unknown("voar".to_owned())));
        assert_eq!(ler(&[]), Err(CommandError::Missing));
    }
}
