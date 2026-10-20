use std::env;
use std::path::{Path, PathBuf};

pub const DEFAULT_SPARKY_PATH: &str = "/usr/share/sparky";
pub const DEFAULT_THEME: &str = "tokyo-night";
const COLORS_FILE: &str = "colors.toml";

pub struct Environment<'a> {
    pub home: &'a str,
    pub config_home: Option<&'a str>,
    pub sparky_path: Option<&'a str>,
}

pub fn sparky_path(value: Option<&str>) -> PathBuf {
    match filled(value) {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from(DEFAULT_SPARKY_PATH),
    }
}

pub fn config_dir(config_home: Option<&str>, home: &str) -> PathBuf {
    match filled(config_home) {
        Some(path) => PathBuf::from(path).join("sparky"),
        None => Path::new(home).join(".config").join("sparky"),
    }
}

pub fn color_files(environment: &Environment) -> Vec<PathBuf> {
    vec![
        config_dir(environment.config_home, environment.home).join(COLORS_FILE),
        sparky_path(environment.sparky_path)
            .join("themes")
            .join(DEFAULT_THEME)
            .join(COLORS_FILE),
    ]
}

pub fn color_files_from_env() -> Vec<PathBuf> {
    let home = env::var("HOME").unwrap_or_default();
    let config_home = env::var("XDG_CONFIG_HOME").ok();
    let sparky = env::var("SPARKY_PATH").ok();
    color_files(&Environment {
        home: &home,
        config_home: config_home.as_deref(),
        sparky_path: sparky.as_deref(),
    })
}

fn filled(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{Environment, color_files, config_dir, sparky_path};
    use std::path::PathBuf;

    fn ambiente<'a>(config_home: Option<&'a str>, sparky: Option<&'a str>) -> Environment<'a> {
        Environment {
            home: "/home/palamar",
            config_home,
            sparky_path: sparky,
        }
    }

    #[test]
    fn sem_sparky_path_usa_o_diretorio_do_sistema() {
        assert_eq!(sparky_path(None), PathBuf::from("/usr/share/sparky"));
        assert_eq!(sparky_path(Some("  ")), PathBuf::from("/usr/share/sparky"));
    }

    #[test]
    fn sparky_path_aponta_para_o_repositorio_em_desenvolvimento() {
        assert_eq!(sparky_path(Some("/src/sparky/data")), PathBuf::from("/src/sparky/data"));
    }

    #[test]
    fn a_configuracao_do_usuario_fica_em_config_sparky() {
        assert_eq!(
            config_dir(None, "/home/palamar"),
            PathBuf::from("/home/palamar/.config/sparky")
        );
        assert_eq!(
            config_dir(Some("/tmp/cfg"), "/home/palamar"),
            PathBuf::from("/tmp/cfg/sparky")
        );
    }

    #[test]
    fn as_cores_do_usuario_vem_antes_das_do_tema_padrao() {
        assert_eq!(
            color_files(&ambiente(None, None)),
            vec![
                PathBuf::from("/home/palamar/.config/sparky/colors.toml"),
                PathBuf::from("/usr/share/sparky/themes/tokyo-night/colors.toml"),
            ]
        );
    }

    #[test]
    fn o_tema_padrao_segue_o_sparky_path() {
        let files = color_files(&ambiente(Some("/x"), Some("/dados")));
        assert_eq!(files[0], PathBuf::from("/x/sparky/colors.toml"));
        assert_eq!(files[1], PathBuf::from("/dados/themes/tokyo-night/colors.toml"));
    }
}
