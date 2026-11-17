use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use sparky_config::{Config, ConfigError, parse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub contents: Option<String>,
    pub stamp: Option<SystemTime>,
}

pub fn path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_default();
    let config_home = env::var("XDG_CONFIG_HOME").ok();
    sparky_config::path(config_home.as_deref(), &home)
}

pub fn read(path: &Path) -> Snapshot {
    Snapshot {
        contents: fs::read_to_string(path).ok(),
        stamp: stamp(path),
    }
}

pub fn migrate(path: &Path) -> io::Result<bool> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Ok(false);
    };
    let Some(updated) = sparky_config::migrate(&contents) else {
        return Ok(false);
    };
    let draft = path.with_extension("toml.novo");
    fs::write(&draft, updated)?;
    fs::rename(&draft, path)?;
    Ok(true)
}

pub fn stamp(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}

pub fn interpret(contents: Option<&str>) -> Result<Config, ConfigError> {
    match contents {
        Some(contents) => parse(contents),
        None => Ok(Config::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::{interpret, migrate};
    use sparky_config::{Config, ConfigError};
    use std::fs;

    #[test]
    fn sem_arquivo_vale_a_configuracao_padrao() {
        assert_eq!(interpret(None), Ok(Config::default()));
    }

    #[test]
    fn arquivo_invalido_vira_erro() {
        assert!(matches!(
            interpret(Some("[general")),
            Err(ConfigError::Invalid(_))
        ));
    }

    #[test]
    fn arquivo_valido_e_lido() {
        let Ok(config) = interpret(Some("[general]\nborder = 3\n")) else {
            panic!("deveria ler");
        };
        assert_eq!(config.general.border, 3);
    }

    #[test]
    fn o_caminho_antigo_do_papel_de_parede_e_regravado_no_arquivo() {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("pasta temporária");
        };
        let arquivo = dir.path().join("sparky.toml");
        let antigo =
            "[general]\nwallpaper = \"/usr/share/sparky/wallpapers/spark/1-aurora.webp\"\n";
        assert!(fs::write(&arquivo, antigo).is_ok());
        assert!(matches!(migrate(&arquivo), Ok(true)));
        assert_eq!(
            fs::read_to_string(&arquivo).ok().as_deref(),
            Some("[general]\nwallpaper = \"/usr/share/sparky/wallpapers/sparky/1-aurora.webp\"\n")
        );
        assert!(!dir.path().join("sparky.toml.novo").exists());
        assert!(matches!(migrate(&arquivo), Ok(false)));
    }

    #[test]
    fn sem_arquivo_nada_e_criado() {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("pasta temporária");
        };
        let arquivo = dir.path().join("sparky.toml");
        assert!(matches!(migrate(&arquivo), Ok(false)));
        assert!(!arquivo.exists());
    }
}
