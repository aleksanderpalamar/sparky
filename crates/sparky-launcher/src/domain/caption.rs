const ELLIPSIS: char = '…';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caption {
    pub text: String,
    pub shortened: bool,
}

pub fn caption(name: &str, width: usize, lines: usize) -> Caption {
    let width = width.max(2);
    let wrapped = wrap(name, width);
    if wrapped.len() <= lines {
        return Caption {
            text: wrapped.join("\n"),
            shortened: false,
        };
    }
    let mut kept: Vec<String> = wrapped.into_iter().take(lines.max(1)).collect();
    if let Some(last) = kept.last_mut() {
        let cut: String = last.chars().take(width - 1).collect();
        *last = format!("{}{ELLIPSIS}", cut.trim_end());
    }
    Caption {
        text: kept.join("\n"),
        shortened: true,
    }
}

fn wrap(name: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in name.split_whitespace().flat_map(|word| pieces(word, width)) {
        let joined = current.chars().count() + 1 + word.chars().count();
        match (current.is_empty(), joined <= width) {
            (true, _) => current = word,
            (false, true) => current = format!("{current} {word}"),
            (false, false) => lines.push(std::mem::replace(&mut current, word)),
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn pieces(word: &str, width: usize) -> Vec<String> {
    let letters: Vec<char> = word.chars().collect();
    letters
        .chunks(width)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::caption;

    #[test]
    fn nome_curto_fica_inteiro() {
        let legenda = caption("Firefox", 14, 2);
        assert_eq!(legenda.text, "Firefox");
        assert!(!legenda.shortened);
    }

    #[test]
    fn nome_de_duas_linhas_quebra_entre_as_palavras() {
        assert_eq!(
            caption("Epic Games Launcher", 14, 2).text,
            "Epic Games\nLauncher"
        );
    }

    #[test]
    fn nome_de_tres_linhas_termina_em_reticencias_na_segunda() {
        let legenda = caption("Advanced Network Configuration", 14, 2);
        assert_eq!(legenda.text, "Advanced\nNetwork…");
        assert!(legenda.shortened);
        let gimp = caption("GNU Image Manipulation Program", 14, 2);
        assert_eq!(gimp.text, "GNU Image\nManipulation…");
    }

    #[test]
    fn palavra_maior_que_a_linha_e_partida() {
        assert_eq!(
            caption("Supercalifragilistic", 10, 2).text,
            "Supercalif\nragilistic"
        );
        assert_eq!(
            caption("Supercalifragilisticoexpialidoso", 10, 2).text,
            "Supercalif\nragilisti…"
        );
    }

    #[test]
    fn acentos_contam_como_uma_letra() {
        assert_eq!(caption("Configurações", 14, 2).text, "Configurações");
    }
}
