use std::path::PathBuf;

use crate::schema::Config;

const FILE: &str = "sparky.toml";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Invalid(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Invalid(message) => write!(formatter, "configuração inválida: {message}"),
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn parse(contents: &str) -> Result<Config, ConfigError> {
    toml::from_str(contents).map_err(|error| ConfigError::Invalid(error.message().to_owned()))
}

pub fn path(config_home: Option<&str>, home: &str) -> PathBuf {
    let base = config_home
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(home).join(".config"));
    base.join("sparky").join(FILE)
}

#[cfg(test)]
mod tests {
    use super::path;
    use std::path::PathBuf;

    #[test]
    fn o_arquivo_fica_na_pasta_do_sparky_em_config() {
        assert_eq!(path(None, "/home/palamar"), PathBuf::from("/home/palamar/.config/sparky/sparky.toml"));
        assert_eq!(path(Some("/cfg"), "/home/palamar"), PathBuf::from("/cfg/sparky/sparky.toml"));
        assert_eq!(path(Some(" "), "/h"), PathBuf::from("/h/.config/sparky/sparky.toml"));
    }
}
