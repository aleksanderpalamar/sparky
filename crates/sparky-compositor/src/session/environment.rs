use std::ffi::OsString;
use std::path::PathBuf;

pub const DESKTOP: &str = "sparky";

pub struct Environment {
    pub wayland_display: OsString,
    pub x_display: Option<u32>,
    pub ipc_socket: Option<PathBuf>,
    pub terminal: String,
}

impl Environment {
    pub fn variables(&self) -> Vec<(String, String)> {
        let mut variables = vec![
            pair("WAYLAND_DISPLAY", self.wayland_display.to_string_lossy()),
            pair("XDG_CURRENT_DESKTOP", DESKTOP),
            pair("XDG_SESSION_DESKTOP", DESKTOP),
            pair("XDG_SESSION_TYPE", "wayland"),
            pair("TERMINAL", self.terminal.as_str()),
        ];
        if let Some(number) = self.x_display {
            variables.push(pair("DISPLAY", format!(":{number}")));
        }
        if let Some(path) = &self.ipc_socket {
            variables.push(pair(sparky_ipc::socket::VARIABLE, path.to_string_lossy()));
        }
        variables
    }
}

pub fn activation_arguments(variables: &[(String, String)]) -> Vec<String> {
    std::iter::once("--systemd".to_owned())
        .chain(variables.iter().map(|(key, value)| format!("{key}={value}")))
        .collect()
}

fn pair(key: &str, value: impl Into<String>) -> (String, String) {
    (key.to_owned(), value.into())
}

#[cfg(test)]
mod tests {
    use super::{Environment, activation_arguments};
    use std::path::PathBuf;

    fn completo() -> Environment {
        Environment {
            wayland_display: "wayland-1".into(),
            x_display: Some(0),
            ipc_socket: Some(PathBuf::from("/run/user/1000/sparky-wayland-1.sock")),
            terminal: "foot".to_owned(),
        }
    }

    #[test]
    fn os_filhos_recebem_a_sessao_do_sparky() {
        let variables = completo().variables();
        assert!(variables.contains(&("WAYLAND_DISPLAY".to_owned(), "wayland-1".to_owned())));
        assert!(variables.contains(&("XDG_CURRENT_DESKTOP".to_owned(), "sparky".to_owned())));
        assert!(variables.contains(&("DISPLAY".to_owned(), ":0".to_owned())));
        assert!(variables.contains(&("SPARKY_SOCKET".to_owned(), "/run/user/1000/sparky-wayland-1.sock".to_owned())));
        assert!(variables.contains(&("TERMINAL".to_owned(), "foot".to_owned())));
    }

    #[test]
    fn sem_xwayland_e_sem_ipc_nao_ha_display_nem_socket() {
        let environment = Environment {
            x_display: None,
            ipc_socket: None,
            ..completo()
        };
        let keys: Vec<String> = environment.variables().into_iter().map(|(key, _)| key).collect();
        assert!(!keys.contains(&"DISPLAY".to_owned()));
        assert!(!keys.contains(&"SPARKY_SOCKET".to_owned()));
    }

    #[test]
    fn a_ativacao_leva_tudo_para_o_systemd_tambem() {
        let arguments = activation_arguments(&[("WAYLAND_DISPLAY".to_owned(), "wayland-1".to_owned())]);
        assert_eq!(arguments, vec!["--systemd", "WAYLAND_DISPLAY=wayland-1"]);
    }
}
