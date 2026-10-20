use std::env;
use std::path::Path;

const TERMINALS: [&str; 5] = ["kitty", "alacritty", "foot", "ghostty", "wezterm"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Apps {
    pub terminal: String,
    pub launcher: String,
    pub files: String,
    pub settings: String,
    pub screenshot: String,
}

impl Apps {
    pub fn detect() -> Self {
        let configured = env::var("TERMINAL").ok();
        Self::with(configured.as_deref(), in_path)
    }

    pub fn with(configured: Option<&str>, present: impl Fn(&str) -> bool) -> Self {
        let configured = configured.map(str::trim).filter(|value| !value.is_empty());
        let terminal = configured
            .map(str::to_owned)
            .or_else(|| TERMINALS.iter().find(|name| present(name)).map(|name| (*name).to_owned()))
            .unwrap_or_else(|| "xterm".to_owned());
        Self {
            terminal,
            launcher: "fuzzel".to_owned(),
            files: "sparky-files".to_owned(),
            settings: "sparky-settings".to_owned(),
            screenshot: "sparky-shot".to_owned(),
        }
    }
}

fn in_path(command: &str) -> bool {
    let path = env::var("PATH").unwrap_or_default();
    path.split(':')
        .filter(|directory| !directory.is_empty())
        .any(|directory| Path::new(directory).join(command).is_file())
}

#[cfg(test)]
mod tests {
    use super::Apps;

    #[test]
    fn o_terminal_configurado_vence() {
        assert_eq!(Apps::with(Some("foot"), |_| true).terminal, "foot");
    }

    #[test]
    fn sem_configuracao_usa_o_primeiro_terminal_instalado() {
        assert_eq!(Apps::with(None, |name| name == "foot" || name == "wezterm").terminal, "foot");
        assert_eq!(Apps::with(Some("  "), |name| name == "kitty").terminal, "kitty");
    }

    #[test]
    fn sem_terminal_nenhum_cai_no_xterm() {
        assert_eq!(Apps::with(None, |_| false).terminal, "xterm");
    }
}
