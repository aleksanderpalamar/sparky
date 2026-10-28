#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct About {
    pub distribution: Option<String>,
    pub kernel: Option<String>,
    pub host: Option<String>,
    pub hardware: super::hardware::Hardware,
}

pub fn pretty_name(os_release: &str) -> Option<String> {
    os_release
        .lines()
        .filter_map(|line| line.split_once('='))
        .find(|(key, _)| key.trim() == "PRETTY_NAME")
        .map(|(_, value)| value.trim().trim_matches('"').to_owned())
        .filter(|name| !name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::pretty_name;

    #[test]
    fn le_o_nome_bonito_da_distribuicao() {
        let release = "NAME=\"Arch Linux\"\nPRETTY_NAME=\"Arch Linux\"\nID=arch\n";
        assert_eq!(pretty_name(release).as_deref(), Some("Arch Linux"));
    }

    #[test]
    fn sem_o_campo_nao_inventa_nome() {
        assert_eq!(pretty_name("ID=arch\n"), None);
        assert_eq!(pretty_name("PRETTY_NAME=\"\"\n"), None);
    }
}
