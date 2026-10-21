use std::env;
use std::path::Path;

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
    use super::installed;

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
}
