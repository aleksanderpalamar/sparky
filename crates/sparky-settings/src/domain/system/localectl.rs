pub fn field(output: &str, label: &str) -> Option<String> {
    output.lines()
        .filter_map(|line| line.trim().split_once(':'))
        .find(|(name, _)| name.trim() == label)
        .map(|(_, value)| value.trim().to_owned())
        .filter(|value| !value.is_empty() && value != "n/a")
}

pub fn locale(output: &str) -> Option<String> {
    let (_, assignments) = output.split_once("System Locale:")?;
    let contents = assignments.lines().take_while(|line| !line.contains(':'))
        .flat_map(str::split_whitespace).collect::<Vec<_>>().join("\n");
    super::key_value::value(&contents, "LANG")
}

pub fn list(output: &str) -> Vec<String> {
    output.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_owned).collect()
}

#[cfg(test)]
mod tests {
    use super::{field, list, locale};

    #[test]
    fn idioma_pode_estar_numa_linha_de_continuacao() {
        assert_eq!(locale("System Locale: LC_TIME=en_US.UTF-8\n LANG=\"pt_BR.UTF-8\"\nVC Keymap: us"), Some("pt_BR.UTF-8".to_owned()));
        assert_eq!(locale("System Locale: n/a\nVC Keymap: br"), None);
    }

    #[test]
    fn campos_ausentes_e_listas_vazias_nao_inventam_valores() {
        assert_eq!(field("VC Keymap: n/a", "VC Keymap"), None);
        assert_eq!(field("VC Keymap:", "VC Keymap"), None);
        assert_eq!(list("\n br \n\nus\n"), ["br", "us"]);
    }
}
