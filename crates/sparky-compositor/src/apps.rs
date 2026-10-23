use std::env;

use sparky_xdg::programs::installed;

const TERMINALS: [&str; 6] = [
    "sparky-terminal",
    "kitty",
    "alacritty",
    "foot",
    "ghostty",
    "wezterm",
];
const LAST_RESORT: &str = "xterm";

pub fn terminal(
    configured: Option<&str>,
    environment: Option<&str>,
    present: impl Fn(&str) -> bool,
) -> String {
    let filled = |value: Option<&str>| {
        value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    };
    filled(configured)
        .or_else(|| filled(environment))
        .or_else(|| {
            TERMINALS
                .iter()
                .find(|name| present(name))
                .map(|name| (*name).to_owned())
        })
        .unwrap_or_else(|| LAST_RESORT.to_owned())
}

pub fn detected_terminal(configured: Option<&str>) -> String {
    let environment = env::var("TERMINAL").ok();
    terminal(configured, environment.as_deref(), installed)
}

#[cfg(test)]
mod tests {
    use super::terminal;

    #[test]
    fn a_configuracao_vence_a_variavel_terminal() {
        assert_eq!(terminal(Some("foot"), Some("alacritty"), |_| true), "foot");
    }

    #[test]
    fn sem_configuracao_vale_a_variavel_terminal() {
        assert_eq!(terminal(None, Some("alacritty"), |_| true), "alacritty");
        assert_eq!(
            terminal(Some(" "), Some("alacritty"), |_| true),
            "alacritty"
        );
    }

    #[test]
    fn sem_nada_usa_o_primeiro_terminal_instalado() {
        assert_eq!(
            terminal(None, None, |name| name == "foot" || name == "wezterm"),
            "foot"
        );
    }

    #[test]
    fn o_terminal_do_sparky_vem_antes_dos_outros_instalados() {
        assert_eq!(
            terminal(None, None, |name| name == "kitty"
                || name == "sparky-terminal"),
            "sparky-terminal"
        );
        assert_eq!(terminal(Some("kitty"), None, |_| true), "kitty");
    }

    #[test]
    fn sem_terminal_nenhum_cai_no_xterm() {
        assert_eq!(terminal(None, None, |_| false), "xterm");
    }
}
