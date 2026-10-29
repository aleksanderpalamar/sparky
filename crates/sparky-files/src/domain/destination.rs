use std::path::{Path, PathBuf};

pub fn resolve(input: &str, current: &Path, home: &Path) -> Option<PathBuf> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    if input == "~" {
        return Some(home.to_path_buf());
    }
    if let Some(rest) = input.strip_prefix("~/") {
        return Some(home.join(rest));
    }
    let path = PathBuf::from(input);
    Some(if path.is_absolute() {
        path
    } else {
        current.join(path)
    })
}

#[cfg(test)]
mod tests {
    use super::resolve;
    use std::path::Path;

    #[test]
    fn destino_relativo_e_til_referem_se_a_pasta_correta() {
        let current = Path::new("/dados/fotos");
        let home = Path::new("/home/palamar");
        assert_eq!(
            resolve("../videos", current, home).as_deref(),
            Some(Path::new("/dados/fotos/../videos"))
        );
        assert_eq!(
            resolve("~/Documentos", current, home).as_deref(),
            Some(Path::new("/home/palamar/Documentos"))
        );
        assert_eq!(resolve("~", current, home).as_deref(), Some(home));
        assert_eq!(
            resolve(" /mnt/usb ", current, home).as_deref(),
            Some(Path::new("/mnt/usb"))
        );
        assert_eq!(resolve("  ", current, home), None);
    }
}
