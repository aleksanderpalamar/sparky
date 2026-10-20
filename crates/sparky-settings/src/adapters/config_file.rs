use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::ports::Store;

pub struct ConfigFile {
    path: PathBuf,
}

impl ConfigFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn from_env() -> Self {
        let home = env::var("HOME").unwrap_or_default();
        let config_home = env::var("XDG_CONFIG_HOME").ok();
        Self::new(sparky_config::path(config_home.as_deref(), &home))
    }
}

impl Store for ConfigFile {
    fn load(&self) -> io::Result<Option<String>> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => Ok(Some(contents)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn save(&self, contents: &str) -> io::Result<()> {
        if let Some(folder) = self.path.parent() {
            fs::create_dir_all(folder)?;
        }
        let draft = self.path.with_extension("toml.novo");
        fs::write(&draft, contents)?;
        fs::rename(&draft, &self.path)
    }

    fn location(&self) -> PathBuf {
        self.path.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::ConfigFile;
    use crate::ports::Store;

    #[test]
    fn sem_arquivo_carrega_nada_e_salvar_cria_a_pasta() {
        let Ok(dir) = tempfile::tempdir() else { panic!("sem pasta temporária") };
        let file = ConfigFile::new(dir.path().join("sparky/sparky.toml"));
        assert_eq!(file.load().ok(), Some(None));
        assert!(file.save("[general]\nborder = 3\n").is_ok());
        assert_eq!(file.load().ok().flatten().as_deref(), Some("[general]\nborder = 3\n"));
        assert!(!dir.path().join("sparky/sparky.toml.novo").exists());
    }
}
