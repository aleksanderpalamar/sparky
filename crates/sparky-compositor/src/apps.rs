use std::env;
use std::path::Path;

const TERMINALS: [&str; 5] = ["kitty", "alacritty", "foot", "ghostty", "wezterm"];
const LAST_RESORT: &str = "xterm";

pub fn terminal(configured: Option<&str>, environment: Option<&str>, present: impl Fn(&str) -> bool) -> String {
    let filled = |value: Option<&str>| value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned);
    filled(configured)
        .or_else(|| filled(environment))
        .or_else(|| TERMINALS.iter().find(|name| present(name)).map(|name| (*name).to_owned()))
        .unwrap_or_else(|| LAST_RESORT.to_owned())
}

pub fn detected_terminal(configured: Option<&str>) -> String {
    let environment = env::var("TERMINAL").ok();
    terminal(configured, environment.as_deref(), installed)
}

pub fn installed(program: &str) -> bool {
    let location = Path::new(program);
    if location.is_absolute() {
        return location.is_file();
    }
    let path = env::var("PATH").unwrap_or_default();
    path.split(':')
        .filter(|directory| !directory.is_empty())
        .any(|directory| Path::new(directory).join(program).is_file())
}

#[cfg(test)]
mod tests {
    use super::{installed, terminal};

    #[test]
    fn caminho_absoluto_vale_se_o_arquivo_existe() {
        let file = std::env::temp_dir().join(format!("sparky-instalado-{}", std::process::id()));
        if std::fs::write(&file, b"").is_err() {
            panic!("não deu para criar o arquivo de teste");
        }
        let found = installed(&file.to_string_lossy());
        let removed = std::fs::remove_file(&file);
        assert!(found);
        assert!(removed.is_ok());
        assert!(!installed(&file.to_string_lossy()));
    }

    #[test]
    fn a_configuracao_vence_a_variavel_terminal() {
        assert_eq!(terminal(Some("foot"), Some("alacritty"), |_| true), "foot");
    }

    #[test]
    fn sem_configuracao_vale_a_variavel_terminal() {
        assert_eq!(terminal(None, Some("alacritty"), |_| true), "alacritty");
        assert_eq!(terminal(Some(" "), Some("alacritty"), |_| true), "alacritty");
    }

    #[test]
    fn sem_nada_usa_o_primeiro_terminal_instalado() {
        assert_eq!(terminal(None, None, |name| name == "foot" || name == "wezterm"), "foot");
    }

    #[test]
    fn sem_terminal_nenhum_cai_no_xterm() {
        assert_eq!(terminal(None, None, |_| false), "xterm");
    }
}
