const DETAIL_INDENT: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub active: bool,
}

pub fn label_of(name: &str) -> &str {
    match name {
        "performance" => "Desempenho",
        "balanced" => "Equilibrado",
        "power-saver" => "Economia de energia",
        other => other,
    }
}

pub fn parse(listing: &str) -> Vec<Profile> {
    listing.lines().filter_map(profile_of).collect()
}

fn profile_of(line: &str) -> Option<Profile> {
    let active = line.starts_with('*');
    let body = line.strip_prefix('*').unwrap_or(line);
    if body.len() - body.trim_start().len() >= DETAIL_INDENT {
        return None;
    }
    let name = body.trim().strip_suffix(':')?;
    if name.is_empty() || name.contains(char::is_whitespace) {
        return None;
    }
    Some(Profile { name: name.to_owned(), active })
}

#[cfg(test)]
mod tests {
    use super::{label_of, parse};

    const LISTING: &str = "* performance:\n    CpuDriver:\tamd_pstate\n    Degraded:   no\n\n  balanced:\n    CpuDriver:\tamd_pstate\n\n  power-saver:\n    CpuDriver:\tamd_pstate\n";

    #[test]
    fn le_os_perfis_na_ordem_e_o_ativo() {
        let profiles = parse(LISTING);
        let names: Vec<&str> = profiles.iter().map(|profile| profile.name.as_str()).collect();
        assert_eq!(names, ["performance", "balanced", "power-saver"]);
        assert!(profiles[0].active);
        assert!(!profiles[1].active);
    }

    #[test]
    fn os_perfis_conhecidos_ganham_nome_em_portugues() {
        assert_eq!(label_of("power-saver"), "Economia de energia");
        assert_eq!(label_of("custom"), "custom");
    }
}
