use std::path::Path;

use crate::message::{CaptureArea, Reply, Request, SessionAction};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    Missing,
    Unknown(String),
    InvalidWorkspace(String),
    InvalidNumber(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::Missing => {
                write!(
                    formatter,
                    "uso: sparky-msg workspaces | outputs | focus-workspace <n> | subscribe | reload | capture output [monitor] <arquivo> | capture window <arquivo> | capture region <x> <y> <largura> <altura> <arquivo> | session lock|suspend|logout|reboot|poweroff"
                )
            }
            CommandError::Unknown(name) => write!(formatter, "comando desconhecido: {name}"),
            CommandError::InvalidWorkspace(value) => write!(formatter, "área inválida: {value}"),
            CommandError::InvalidNumber(value) => write!(formatter, "número inválido: {value}"),
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
        "reload" => Ok(Request::ReloadConfig),
        "outputs" => Ok(Request::Outputs),
        "capture" => capture(&arguments[1..]),
        "focus-workspace" => workspace(arguments.get(1)),
        "session" => session(arguments.get(1)),
        other => Err(CommandError::Unknown(other.to_owned())),
    }
}

fn session(name: Option<&String>) -> Result<Request, CommandError> {
    let name = name.ok_or(CommandError::Missing)?;
    let action = SessionAction::named(name).ok_or_else(|| CommandError::Unknown(name.clone()))?;
    Ok(Request::Session { action })
}

pub fn with_absolute_path(request: Request, directory: &Path) -> Request {
    match request {
        Request::Capture { area, path } => Request::Capture {
            path: directory.join(path).to_string_lossy().into_owned(),
            area,
        },
        other => other,
    }
}

pub fn succeeded(reply: &Reply) -> bool {
    !matches!(reply, Reply::Error { .. })
}

fn capture(arguments: &[String]) -> Result<Request, CommandError> {
    let texts: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let (area, path) = match texts.as_slice() {
        ["output", path] => (CaptureArea::Output { output: None }, path),
        ["output", name, path] => (CaptureArea::Output { output: Some((*name).to_owned()) }, path),
        ["window", path] => (CaptureArea::Window, path),
        ["region", x, y, width, height, path] => (region(x, y, width, height)?, path),
        _ => return Err(CommandError::Missing),
    };
    Ok(Request::Capture { area, path: (*path).to_owned() })
}

fn region(x: &str, y: &str, width: &str, height: &str) -> Result<CaptureArea, CommandError> {
    let number = |text: &str| text.parse::<i32>().map_err(|_| CommandError::InvalidNumber(text.to_owned()));
    Ok(CaptureArea::Region {
        output: None,
        x: number(x)?,
        y: number(y)?,
        width: number(width)?,
        height: number(height)?,
    })
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
    use super::{CommandError, parse, succeeded};
    use crate::message::{Reply, Request};

    #[test]
    fn le_as_tres_formas_de_captura() {
        use crate::message::CaptureArea;
        let capture = |area: CaptureArea| Ok(Request::Capture { area, path: "a.png".to_owned() });
        assert_eq!(ler(&["capture", "output", "a.png"]), capture(CaptureArea::Output { output: None }));
        assert_eq!(
            ler(&["capture", "output", "DP-1", "a.png"]),
            capture(CaptureArea::Output { output: Some("DP-1".to_owned()) })
        );
        assert_eq!(ler(&["capture", "window", "a.png"]), capture(CaptureArea::Window));
        assert_eq!(
            ler(&["capture", "region", "1", "2", "3", "4", "a.png"]),
            capture(CaptureArea::Region { output: None, x: 1, y: 2, width: 3, height: 4 })
        );
    }

    #[test]
    fn captura_incompleta_ou_com_numero_errado_e_erro() {
        assert_eq!(ler(&["capture", "region", "1", "2", "a.png"]), Err(CommandError::Missing));
        assert_eq!(
            ler(&["capture", "region", "1", "x", "3", "4", "a.png"]),
            Err(CommandError::InvalidNumber("x".to_owned()))
        );
    }

    #[test]
    fn o_caminho_relativo_vira_absoluto_e_o_absoluto_fica() {
        use super::with_absolute_path;
        use crate::message::CaptureArea;
        let pedido = |path: &str| Request::Capture { area: CaptureArea::Window, path: path.to_owned() };
        let pasta = std::path::Path::new("/home/palamar");
        assert_eq!(with_absolute_path(pedido("a.png"), pasta), pedido("/home/palamar/a.png"));
        assert_eq!(with_absolute_path(pedido("/tmp/b.png"), pasta), pedido("/tmp/b.png"));
        assert_eq!(with_absolute_path(Request::Workspaces, pasta), Request::Workspaces);
    }

    #[test]
    fn resposta_de_erro_e_falha_para_o_script() {
        assert!(succeeded(&Reply::Done));
        assert!(!succeeded(&Reply::Error { message: "x".to_owned() }));
    }

    fn ler(argumentos: &[&str]) -> Result<Request, CommandError> {
        let argumentos: Vec<String> = argumentos.iter().map(|a| (*a).to_owned()).collect();
        parse(&argumentos)
    }

    #[test]
    fn le_os_tres_comandos() {
        assert_eq!(ler(&["workspaces"]), Ok(Request::Workspaces));
        assert_eq!(ler(&["subscribe"]), Ok(Request::Subscribe));
        assert_eq!(ler(&["focus-workspace", "4"]), Ok(Request::FocusWorkspace { id: 4 }));
        assert_eq!(ler(&["reload"]), Ok(Request::ReloadConfig));
        assert_eq!(ler(&["outputs"]), Ok(Request::Outputs));
    }

    #[test]
    fn le_as_acoes_de_sessao() {
        use crate::message::SessionAction;
        let sessao = |action: SessionAction| Ok(Request::Session { action });
        assert_eq!(ler(&["session", "lock"]), sessao(SessionAction::Lock));
        assert_eq!(ler(&["session", "suspend"]), sessao(SessionAction::Suspend));
        assert_eq!(ler(&["session", "logout"]), sessao(SessionAction::Logout));
        assert_eq!(ler(&["session", "reboot"]), sessao(SessionAction::Reboot));
        assert_eq!(ler(&["session", "poweroff"]), sessao(SessionAction::Poweroff));
        assert_eq!(ler(&["session"]), Err(CommandError::Missing));
        assert_eq!(ler(&["session", "voar"]), Err(CommandError::Unknown("voar".to_owned())));
    }

    #[test]
    fn recusa_area_invalida_e_comando_desconhecido() {
        assert_eq!(ler(&["focus-workspace", "x"]), Err(CommandError::InvalidWorkspace("x".to_owned())));
        assert_eq!(ler(&["focus-workspace"]), Err(CommandError::Missing));
        assert_eq!(ler(&["voar"]), Err(CommandError::Unknown("voar".to_owned())));
        assert_eq!(ler(&[]), Err(CommandError::Missing));
    }
}
