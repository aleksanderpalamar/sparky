use std::path::Path;

pub fn tilde(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::tilde;
    use std::path::Path;

    #[test]
    fn o_caminho_dentro_do_home_comeca_com_til() {
        let home = Path::new("/home/p");
        assert_eq!(tilde(Path::new("/home/p"), home), "~");
        assert_eq!(tilde(Path::new("/home/p/Fotos/2026"), home), "~/Fotos/2026");
        assert_eq!(tilde(Path::new("/etc"), home), "/etc");
    }
}
