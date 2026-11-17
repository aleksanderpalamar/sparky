use crate::session::environment::{Environment, activation_arguments};
use crate::spawn::start;

pub const TARGET: &str = "sparky-session.target";
const ACTIVATION: &str = "dbus-update-activation-environment";
const SYSTEMCTL: &str = "systemctl";
const PORTALS: &str = "xdg-desktop-portal*.service";

pub fn preparation(variables: &[(String, String)]) -> Vec<(&'static str, Vec<String>)> {
    let stop = ["--user", "stop", PORTALS].map(str::to_owned).to_vec();
    let restart = ["--user", "restart", TARGET].map(str::to_owned).to_vec();
    vec![
        (ACTIVATION, activation_arguments(variables)),
        (SYSTEMCTL, stop),
        (SYSTEMCTL, restart),
    ]
}

pub fn prepare(environment: &Environment) {
    for (program, arguments) in preparation(&environment.variables()) {
        let finished = start(program, &arguments, environment).and_then(|mut child| child.wait());
        match finished {
            Ok(status) if status.success() => {}
            Ok(status) => tracing::warn!("sessão: {program} {arguments:?} terminou com {status}"),
            Err(error) => tracing::warn!("sessão: não deu para rodar {program}: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TARGET, preparation};

    #[test]
    fn para_os_portais_da_sessao_anterior_antes_de_ligar_a_nova() {
        let variaveis = vec![("WAYLAND_DISPLAY".to_owned(), "wayland-1".to_owned())];
        let texto = |lista: &[&str]| {
            lista
                .iter()
                .map(|item| (*item).to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            preparation(&variaveis),
            vec![
                (
                    "dbus-update-activation-environment",
                    texto(&["--systemd", "WAYLAND_DISPLAY=wayland-1"])
                ),
                (
                    "systemctl",
                    texto(&["--user", "stop", "xdg-desktop-portal*.service"])
                ),
                ("systemctl", texto(&["--user", "restart", TARGET])),
            ]
        );
    }

    #[test]
    fn o_alvo_e_o_da_sessao_do_sparky() {
        assert_eq!(TARGET, "sparky-session.target");
    }
}
