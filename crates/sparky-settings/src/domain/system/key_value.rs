pub fn value(contents: &str, key: &str) -> Option<String> {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .find(|(name, _)| name.trim() == key)
        .map(|(_, value)| unquote(value.trim()).to_owned())
}

pub fn flag(contents: &str, key: &str) -> bool {
    matches!(value(contents, key).as_deref(), Some("yes" | "true" | "1" | "on"))
}

fn unquote(value: &str) -> &str {
    let double = value.strip_prefix('"').and_then(|inner| inner.strip_suffix('"'));
    let single = value.strip_prefix('\'').and_then(|inner| inner.strip_suffix('\''));
    double.or(single).unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::{flag, value};

    const SHOW: &str = "Timezone=America/Sao_Paulo\nLocalRTC=no\nNTP=yes\nLANG=\"pt_BR.UTF-8\"\n";

    #[test]
    fn le_valores_com_e_sem_aspas() {
        assert_eq!(value(SHOW, "Timezone").as_deref(), Some("America/Sao_Paulo"));
        assert_eq!(value(SHOW, "LANG").as_deref(), Some("pt_BR.UTF-8"));
        assert_eq!(value(SHOW, "Falta"), None);
    }

    #[test]
    fn le_marcas_de_sim_e_nao() {
        assert!(flag(SHOW, "NTP"));
        assert!(!flag(SHOW, "LocalRTC"));
    }
}
