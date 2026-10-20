use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserDir {
    Desktop,
    Documents,
    Download,
    Music,
    Pictures,
    Videos,
}

impl UserDir {
    fn key(self) -> &'static str {
        match self {
            UserDir::Desktop => "XDG_DESKTOP_DIR",
            UserDir::Documents => "XDG_DOCUMENTS_DIR",
            UserDir::Download => "XDG_DOWNLOAD_DIR",
            UserDir::Music => "XDG_MUSIC_DIR",
            UserDir::Pictures => "XDG_PICTURES_DIR",
            UserDir::Videos => "XDG_VIDEOS_DIR",
        }
    }

    fn fallback(self) -> &'static str {
        match self {
            UserDir::Desktop => "Desktop",
            UserDir::Documents => "Documents",
            UserDir::Download => "Downloads",
            UserDir::Music => "Music",
            UserDir::Pictures => "Pictures",
            UserDir::Videos => "Videos",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserDirs {
    home: String,
    contents: Option<String>,
}

impl UserDirs {
    pub fn new(home: impl Into<String>, contents: Option<String>) -> Self {
        Self {
            home: home.into(),
            contents,
        }
    }

    pub fn from_env() -> Option<Self> {
        let home = env::var("HOME").ok()?;
        let config = env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{home}/.config"));
        let contents = fs::read_to_string(Path::new(&config).join("user-dirs.dirs")).ok();
        Some(Self::new(home, contents))
    }

    pub fn home(&self) -> PathBuf {
        PathBuf::from(&self.home)
    }

    pub fn get(&self, dir: UserDir) -> PathBuf {
        self.contents
            .as_deref()
            .and_then(|contents| self.configured(contents, dir.key()))
            .unwrap_or_else(|| self.home().join(dir.fallback()))
    }

    fn configured(&self, contents: &str, key: &str) -> Option<PathBuf> {
        let value = contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#'))
            .filter_map(|line| line.split_once('='))
            .find(|(name, _)| name.trim() == key)
            .map(|(_, value)| value.trim().trim_matches('"'))?;
        let expanded = value.replace("$HOME", &self.home);
        (!expanded.is_empty()).then(|| PathBuf::from(expanded))
    }
}

#[cfg(test)]
mod tests {
    use super::{UserDir, UserDirs};
    use std::path::PathBuf;

    fn pastas(contents: Option<&str>) -> UserDirs {
        UserDirs::new("/home/palamar", contents.map(str::to_owned))
    }

    #[test]
    fn le_a_pasta_configurada_trocando_o_home() {
        let contents = "XDG_DESKTOP_DIR=\"$HOME/Área de trabalho\"\nXDG_PICTURES_DIR=\"$HOME/Imagens\"\n";
        assert_eq!(pastas(Some(contents)).get(UserDir::Pictures), PathBuf::from("/home/palamar/Imagens"));
        assert_eq!(pastas(Some(contents)).get(UserDir::Desktop), PathBuf::from("/home/palamar/Área de trabalho"));
    }

    #[test]
    fn caminho_absoluto_fica_como_esta() {
        assert_eq!(pastas(Some("XDG_MUSIC_DIR=\"/dados/musica\"\n")).get(UserDir::Music), PathBuf::from("/dados/musica"));
    }

    #[test]
    fn sem_configuracao_vale_o_nome_padrao_na_pasta_pessoal() {
        assert_eq!(pastas(None).get(UserDir::Download), PathBuf::from("/home/palamar/Downloads"));
        assert_eq!(pastas(Some("# nada\n")).get(UserDir::Pictures), PathBuf::from("/home/palamar/Pictures"));
    }

    #[test]
    fn linha_comentada_e_ignorada() {
        let contents = "# XDG_VIDEOS_DIR=\"$HOME/Velho\"\nXDG_VIDEOS_DIR=\"$HOME/Vídeos\"\n";
        assert_eq!(pastas(Some(contents)).get(UserDir::Videos), PathBuf::from("/home/palamar/Vídeos"));
    }

    #[test]
    fn a_pasta_pessoal_e_o_home() {
        assert_eq!(pastas(None).home(), PathBuf::from("/home/palamar"));
    }
}
