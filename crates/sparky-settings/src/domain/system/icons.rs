const KEY: &str = "gtk-icon-theme-name";
const SECTION: &str = "[Settings]";

pub fn icon_theme_of(contents: &str) -> Option<String> {
    let name = contents.lines().map(str::trim).find(|line| !line.is_empty())?;
    let allowed = name.chars().all(|letter| letter.is_ascii_alphanumeric() || matches!(letter, '-' | '_' | '.' | '+'));
    allowed.then(|| name.to_owned())
}

pub fn with_icon_theme(settings: &str, name: &str) -> String {
    let line = format!("{KEY}={name}");
    let mut lines: Vec<String> = settings.lines().map(str::to_owned).collect();
    let Some(start) = lines.iter().position(|text| text.trim() == SECTION) else {
        lines.extend([SECTION.to_owned(), line]);
        return joined(&lines);
    };
    let body = start + 1;
    let end = lines.iter().skip(body).position(|text| text.trim().starts_with('[')).map_or(lines.len(), |offset| body + offset);
    let existing = lines.iter().take(end).skip(body).position(|text| text.split_once('=').is_some_and(|(key, _)| key.trim() == KEY));
    match existing {
        Some(offset) => lines[body + offset] = line,
        None => lines.insert(body, line),
    }
    joined(&lines)
}

fn joined(lines: &[String]) -> String {
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use super::{icon_theme_of, with_icon_theme};

    #[test]
    fn o_nome_dos_icones_vem_da_primeira_linha() {
        assert_eq!(icon_theme_of("\nYaru-blue\n").as_deref(), Some("Yaru-blue"));
        assert_eq!(icon_theme_of("Yaru sage"), None);
        assert_eq!(icon_theme_of("Yaru\nreal=1"), Some("Yaru".to_owned()));
        assert_eq!(icon_theme_of(""), None);
    }

    #[test]
    fn sem_arquivo_cria_a_secao() {
        assert_eq!(with_icon_theme("", "Yaru-blue"), "[Settings]\ngtk-icon-theme-name=Yaru-blue\n");
    }

    #[test]
    fn troca_so_a_chave_dos_icones_e_guarda_o_resto() {
        let antes = "[Settings]\ngtk-theme-name=Adwaita-dark\ngtk-icon-theme-name = Papirus\n";
        assert_eq!(with_icon_theme(antes, "Yaru-sage"), "[Settings]\ngtk-theme-name=Adwaita-dark\ngtk-icon-theme-name=Yaru-sage\n");
    }

    #[test]
    fn a_chave_de_outra_secao_nao_conta() {
        let antes = "[Outra]\ngtk-icon-theme-name=X\n[Settings]\ngtk-font-name=Sans 10\n";
        let depois = "[Outra]\ngtk-icon-theme-name=X\n[Settings]\ngtk-icon-theme-name=Yaru-red\ngtk-font-name=Sans 10\n";
        assert_eq!(with_icon_theme(antes, "Yaru-red"), depois);
    }
}
