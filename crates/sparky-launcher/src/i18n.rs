use std::sync::OnceLock;

use fluent_bundle::FluentArgs;
use sparky_i18n::Catalog;

const PORTUGUESE: &str = include_str!("../../../data/i18n/pt-BR/launcher.ftl");
const ENGLISH: &str = include_str!("../../../data/i18n/en/launcher.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

#[cfg(test)]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::new(&sparky_i18n::Locale::parse("pt-BR"), PORTUGUESE, ENGLISH)
}

#[cfg(not(test))]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::for_domain("launcher", PORTUGUESE, ENGLISH)
}

pub fn text(key: &str) -> String {
    message(key, None)
}

pub fn with_group(key: &str, group: &str) -> String {
    let mut args = FluentArgs::new();
    args.set("group", group.to_owned());
    message(key, Some(&args))
}

fn message(key: &str, args: Option<&FluentArgs<'_>>) -> String {
    CATALOG
        .get_or_init(load_catalog)
        .as_ref()
        .ok()
        .and_then(|catalog| catalog.message(key, args))
        .unwrap_or_else(|| key.to_owned())
}

#[cfg(test)]
mod tests {
    use sparky_i18n::{Catalog, Locale};

    use super::{ENGLISH, PORTUGUESE};

    #[test]
    fn traduz_menu_de_sessao() {
        let english = Catalog::new(&Locale::parse("en-US"), PORTUGUESE, ENGLISH).unwrap();
        assert_eq!(english.text("lock-name").as_deref(), Some("Lock"));
        assert_eq!(
            english.text("no-apps").as_deref(),
            Some("No applications found")
        );
    }

    #[test]
    fn o_nome_do_grupo_entra_no_item_do_menu() {
        assert_eq!(
            super::with_group("menu-add-to", "Jogos"),
            "Adicionar a Jogos"
        );
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
