use std::env;
use std::fs;
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

pub fn stamp(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|metadata| metadata.modified()).ok()
}

pub fn interpret(contents: Option<&str>) -> Result<Config, ConfigError> {
    match contents {
        Some(contents) => parse(contents),
        None => Ok(Config::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::interpret;
    use sparky_config::{Config, ConfigError};

    #[test]
    fn sem_arquivo_vale_a_configuracao_padrao() {
        assert_eq!(interpret(None), Ok(Config::default()));
    }

    #[test]
    fn arquivo_invalido_vira_erro() {
        assert!(matches!(interpret(Some("[general")), Err(ConfigError::Invalid(_))));
    }

    #[test]
    fn arquivo_valido_e_lido() {
        let Ok(config) = interpret(Some("[general]\nborder = 3\n")) else {
            panic!("deveria ler");
        };
        assert_eq!(config.general.border, 3);
    }
}
