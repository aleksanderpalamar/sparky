pub fn value<'a>(contents: &'a str, key: &str) -> Option<&'a str> {
    contents
        .lines()
        .filter_map(|line| assignment(line.trim()))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| value)
}

fn assignment(line: &str) -> Option<(&str, &str)> {
    if line.starts_with('#') {
        return None;
    }
    let (name, value) = line.split_once('=')?;
    Some((name.trim(), unquote(value.trim())))
}

fn unquote(value: &str) -> &str {
    let double = value.strip_prefix('"').and_then(|v| v.strip_suffix('"'));
    let single = value.strip_prefix('\'').and_then(|v| v.strip_suffix('\''));
    double.or(single).unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::value;

    const CORES: &str = "mode = \"dark\"\n\naccent = \"#7aa2f7\"\n# red = \"#000000\"\nred = '#f7768e'\n";

    #[test]
    fn le_o_valor_entre_aspas_duplas() {
        assert_eq!(value(CORES, "accent"), Some("#7aa2f7"));
    }

    #[test]
    fn le_o_valor_entre_aspas_simples() {
        assert_eq!(value(CORES, "red"), Some("#f7768e"));
    }

    #[test]
    fn ignora_linhas_comentadas() {
        assert_eq!(value("# accent = \"#000000\"\n", "accent"), None);
    }

    #[test]
    fn chave_ausente_devolve_nada() {
        assert_eq!(value(CORES, "magenta"), None);
    }

    #[test]
    fn nao_confunde_chave_com_prefixo_de_outra() {
        assert_eq!(value("bright_red = \"#ff7a93\"\n", "red"), None);
    }
}
