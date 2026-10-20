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
        Self::with(configured.as_deref(), installed)
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
    use super::{Apps, installed};

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
