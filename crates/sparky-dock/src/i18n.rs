use std::sync::OnceLock;

use sparky_i18n::Catalog;

const PORTUGUESE: &str = include_str!("../../../data/i18n/pt-BR/dock.ftl");
const ENGLISH: &str = include_str!("../../../data/i18n/en/dock.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

#[cfg(test)]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::new(&sparky_i18n::Locale::parse("pt-BR"), PORTUGUESE, ENGLISH)
}

#[cfg(not(test))]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::for_domain("dock", PORTUGUESE, ENGLISH)
}

pub fn text(key: &str) -> String {
    CATALOG
        .get_or_init(load_catalog)
        .as_ref()
        .ok()
        .and_then(|catalog| catalog.text(key))
        .unwrap_or_else(|| key.to_owned())
}

#[cfg(test)]
mod tests {
    use sparky_i18n::{Catalog, Locale};

    use super::{ENGLISH, PORTUGUESE};

    #[test]
    fn traduz_o_dialogo_para_o_ingles() {
        let Ok(english) = Catalog::new(&Locale::parse("en-US"), PORTUGUESE, ENGLISH) else {
            panic!("catálogo inválido");
        };
        assert_eq!(english.text("picker-add").as_deref(), Some("Add selected"));
        assert_eq!(super::text("picker-cancel"), "Cancelar");
    }

    #[test]
    fn todas_as_chaves_do_portugues_existem_em_ingles() {
        let chaves = |texto: &str| -> Vec<String> {
            texto
                .lines()
                .filter_map(|linha| linha.split_once(" = ").map(|(chave, _)| chave.to_owned()))
                .collect()
        };
        assert_eq!(chaves(PORTUGUESE), chaves(ENGLISH));
    }
}
