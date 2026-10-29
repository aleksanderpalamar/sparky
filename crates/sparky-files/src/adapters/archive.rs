use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use crate::ports::Archiver;

pub struct TarArchive;

impl Archiver for TarArchive {
    fn create(&self, paths: &[PathBuf], target: &Path) -> io::Result<()> {
        let source_dir = paths
            .first()
            .and_then(|path| path.parent())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "nenhum item selecionado")
            })?;
        let names: Vec<_> = paths
            .iter()
            .map(|path| {
                let name = path.strip_prefix(source_dir).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "pastas de origem diferentes")
                })?;
                if name.components().count() != 1
                    || !matches!(name.components().next(), Some(Component::Normal(_)))
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "nome de origem inválido",
                    ));
                }
                Ok(name.to_path_buf())
            })
            .collect::<io::Result<_>>()?;
        let target_dir = target
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "destino inválido"))?;
        let temporary = tempfile::NamedTempFile::new_in(target_dir)?;
        let result = Command::new("tar")
            .arg("-czf")
            .arg(temporary.path())
            .arg("-C")
            .arg(source_dir)
            .arg("--")
            .args(&names)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !result.success() {
            return Err(io::Error::other("não foi possível compactar os itens"));
        }
        temporary
            .persist_noclobber(target)
            .map_err(|error| error.error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::TarArchive;
    use crate::ports::Archiver;
    use std::fs;
    use std::process::Command;

    #[test]
    fn compacta_pasta_e_arquivo_sem_sobrescrever_destino() {
        let root = tempfile::tempdir().unwrap();
        let folder = root.path().join("Pasta");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("dado.txt"), "oi").unwrap();
        let file = root.path().join("outro.txt");
        fs::write(&file, "olá").unwrap();
        let archive = root.path().join("itens.tar.gz");
        TarArchive.create(&[folder, file], &archive).unwrap();
        let listing = Command::new("tar")
            .arg("-tzf")
            .arg(&archive)
            .output()
            .unwrap();
        assert!(listing.status.success());
        let names = String::from_utf8(listing.stdout).unwrap();
        assert!(names.contains("Pasta/dado.txt"));
        assert!(names.contains("outro.txt"));
        assert!(
            TarArchive
                .create(&[root.path().join("outro.txt")], &archive)
                .is_err()
        );
    }
}
