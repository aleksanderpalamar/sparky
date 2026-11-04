use std::sync::OnceLock;

use fluent_bundle::FluentArgs;
use sparky_i18n::Catalog;

const PORTUGUESE: &str = include_str!("../../../data/i18n/pt-BR/login.ftl");
const ENGLISH: &str = include_str!("../../../data/i18n/en/login.ftl");

static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

pub fn text(key: &str) -> String {
    message(key, None)
}

pub fn message(key: &str, args: Option<&FluentArgs<'_>>) -> String {
    CATALOG
        .get_or_init(|| Catalog::for_domain("login", PORTUGUESE, ENGLISH))
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
    fn a_tela_de_entrada_tem_textos_nos_dois_idiomas() {
        let portuguese = Catalog::new(&Locale::parse("pt-BR"), PORTUGUESE, ENGLISH).unwrap();
        let english = Catalog::new(&Locale::parse("en-US"), PORTUGUESE, ENGLISH).unwrap();
        assert_eq!(
            portuguese.text("power-shutdown").as_deref(),
            Some("Desligar")
        );
        assert_eq!(english.text("power-shutdown").as_deref(), Some("Shut down"));
        assert_eq!(english.text("unlock").as_deref(), Some("Unlock"));
    }
}
