#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Creation {
    Folder(String),
    File(String),
}

pub fn valid(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\0'])
}

pub fn creation(input: &str) -> Option<Creation> {
    let input = input.trim();
    let created = match input.strip_suffix('/') {
        Some(folder) => Creation::Folder(folder.trim_end_matches('/').to_owned()),
        None => Creation::File(input.to_owned()),
    };
    let (Creation::Folder(name) | Creation::File(name)) = &created;
    valid(name).then_some(created)
}

pub fn free_name(name: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(name) {
        return name.to_owned();
    }
    let (stem, extension) = split(name);
    (1..=u32::MAX)
        .map(|number| match number {
            1 => format!("{stem} (cópia){extension}"),
            _ => format!("{stem} (cópia {number}){extension}"),
        })
        .find(|candidate| !taken(candidate))
        .unwrap_or_else(|| name.to_owned())
}

pub fn stem_length(name: &str) -> usize {
    split(name).0.chars().count()
}

fn split(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(dot) if dot > 0 => name.split_at(dot),
        _ => (name, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::{Creation, creation, free_name, stem_length, valid};

    #[test]
    fn barra_no_fim_cria_pasta_e_sem_ela_cria_arquivo() {
        assert_eq!(creation(" Projetos/ "), Some(Creation::Folder("Projetos".to_owned())));
        assert_eq!(creation("notas.md"), Some(Creation::File("notas.md".to_owned())));
    }

    #[test]
    fn nomes_que_o_sistema_nao_aceita_sao_recusados() {
        assert_eq!(creation(""), None);
        assert_eq!(creation("/"), None);
        assert_eq!(creation("a/b"), None);
        assert_eq!(creation("../"), None);
        assert!(!valid("."));
        assert!(valid(".bashrc"));
    }

    #[test]
    fn nome_livre_fica_como_esta() {
        assert_eq!(free_name("foto.png", |_| false), "foto.png");
    }

    #[test]
    fn nome_ocupado_ganha_copia_antes_da_extensao() {
        let ocupados = ["foto.png", "foto (cópia).png", "foto (cópia 2).png"];
        assert_eq!(free_name("foto.png", |nome| ocupados.contains(&nome)), "foto (cópia 3).png");
        assert_eq!(free_name("Fotos", |nome| nome == "Fotos"), "Fotos (cópia)");
        assert_eq!(free_name(".bashrc", |nome| nome == ".bashrc"), ".bashrc (cópia)");
    }

    #[test]
    fn ao_renomear_seleciona_o_nome_sem_a_extensao() {
        assert_eq!(stem_length("relatório.pdf"), 9);
        assert_eq!(stem_length("pacote.tar.gz"), 10);
        assert_eq!(stem_length(".bashrc"), 7);
        assert_eq!(stem_length("Fotos"), 5);
    }
}
