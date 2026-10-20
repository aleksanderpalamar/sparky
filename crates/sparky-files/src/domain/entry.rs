use chrono::NaiveDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Directory,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub kind: Kind,
    pub size: u64,
    pub modified: Option<NaiveDateTime>,
}

impl Entry {
    pub fn hidden(&self) -> bool {
        self.name.starts_with('.')
    }

    pub fn is_directory(&self) -> bool {
        self.kind == Kind::Directory
    }
}

pub fn sort(entries: &mut [Entry]) {
    entries.sort_by_cached_key(|entry| (!entry.is_directory(), entry.name.to_lowercase(), entry.name.clone()));
}

#[cfg(test)]
pub fn entry(name: &str, kind: Kind) -> Entry {
    Entry {
        name: name.to_owned(),
        kind,
        size: 0,
        modified: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{Kind, entry, sort};

    #[test]
    fn pastas_vem_antes_e_o_resto_em_ordem_alfabetica_sem_caixa() {
        let mut entries = vec![
            entry("zeta.txt", Kind::File),
            entry("Beta", Kind::Directory),
            entry("alfa.txt", Kind::File),
            entry("alfa", Kind::Directory),
        ];
        sort(&mut entries);
        let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(names, ["alfa", "Beta", "alfa.txt", "zeta.txt"]);
    }

    #[test]
    fn nome_com_ponto_na_frente_e_oculto() {
        assert!(entry(".config", Kind::Directory).hidden());
        assert!(!entry("config", Kind::Directory).hidden());
    }
}
