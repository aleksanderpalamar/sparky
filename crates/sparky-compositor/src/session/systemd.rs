use crate::session::environment::{Environment, activation_arguments};
use crate::spawn::start;

pub const TARGET: &str = "sparky-session.target";
const ACTIVATION: &str = "dbus-update-activation-environment";
const SYSTEMCTL: &str = "systemctl";

pub fn preparation(variables: &[(String, String)]) -> Vec<(&'static str, Vec<String>)> {
    let restart = ["--user", "restart", TARGET].map(str::to_owned).to_vec();
    vec![
        (ACTIVATION, activation_arguments(variables)),
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
    fn exporta_o_ambiente_antes_de_reiniciar_a_sessao_grafica() {
        let variaveis = vec![("WAYLAND_DISPLAY".to_owned(), "wayland-1".to_owned())];
        let passos = preparation(&variaveis);
        assert_eq!(
            passos,
            vec![
                (
                    "dbus-update-activation-environment",
                    vec![
                        "--systemd".to_owned(),
                        "WAYLAND_DISPLAY=wayland-1".to_owned()
                    ]
                ),
                (
                    "systemctl",
                    vec!["--user".to_owned(), "restart".to_owned(), TARGET.to_owned()]
                ),
            ]
        );
    }

    #[test]
    fn o_alvo_e_o_da_sessao_do_sparky() {
        assert_eq!(TARGET, "sparky-session.target");
    }
}
