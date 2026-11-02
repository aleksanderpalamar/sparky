use std::fs;
use std::io;
use std::path::Path;

use crate::domain::memory::Memory;

const TEMPORARY: &str = "tmp";

pub fn load(path: &Path) -> Memory {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save(path: &Path, memory: &Memory) -> io::Result<()> {
    let text = serde_json::to_string(memory).map_err(io::Error::other)?;
    let temporary = path.with_extension(TEMPORARY);
    fs::write(&temporary, text)?;
    fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::{load, save};
    use crate::domain::memory::Memory;

    #[test]
    fn grava_e_le_o_ultimo_login() {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("pasta temporária");
        };
        let path = dir.path().join("memory.json");
        let memory = Memory {
            last_user: Some("palamar".into()),
            last_session: Some("sparky".into()),
        };
        assert!(save(&path, &memory).is_ok());
        assert_eq!(load(&path), memory);
        assert!(!path.with_extension("tmp").exists());
    }

    #[test]
    fn arquivo_ausente_ou_estragado_vale_memoria_vazia() {
        let Ok(dir) = tempfile::tempdir() else {
            panic!("pasta temporária");
        };
        let path = dir.path().join("memory.json");
        assert_eq!(load(&path), Memory::default());
        assert!(std::fs::write(&path, "{quebrado").is_ok());
        assert_eq!(load(&path), Memory::default());
    }
}
