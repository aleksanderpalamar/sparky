use sparky_ui::natural::fold;

use crate::domain::entry::Entry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    text: String,
    words: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finish {
    Complete,
    Limited,
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum SearchEvent {
    Found(Vec<Entry>),
    Finished(Finish),
}

impl Query {
    pub fn new(text: &str) -> Option<Self> {
        let words: Vec<String> = fold(text).split_whitespace().map(str::to_owned).collect();
        (!words.is_empty()).then(|| Self {
            text: text.trim().to_owned(),
            words,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn matches(&self, name: &str) -> bool {
        let folded = fold(name);
        self.words.iter().all(|word| folded.contains(word.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::Query;

    #[test]
    fn todas_as_palavras_precisam_aparecer_sem_caixa_nem_acento() {
        let Some(busca) = Query::new("  foto  PRAIA ") else {
            panic!("a busca deveria valer")
        };
        assert_eq!(busca.text(), "foto  PRAIA");
        assert!(busca.matches("Praia - Fóto 1.jpg"));
        assert!(!busca.matches("foto.jpg"));
        assert!(Query::new("coração").is_some_and(|busca| busca.matches("CORACAO.txt")));
    }

    #[test]
    fn busca_vazia_nao_vale() {
        assert_eq!(Query::new("   "), None);
    }
}
