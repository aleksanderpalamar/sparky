use std::sync::OnceLock;

use fluent_bundle::FluentArgs;
use sparky_i18n::Catalog;

const PORTUGUESE: &str = include_str!("../../../data/i18n/pt-BR/polkit.ftl");
const ENGLISH: &str = include_str!("../../../data/i18n/en/polkit.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

pub fn text(key: &str) -> String {
    message(key, None)
}

pub fn message(key: &str, args: Option<&FluentArgs<'_>>) -> String {
    CATALOG
        .get_or_init(|| Catalog::for_domain("polkit", PORTUGUESE, ENGLISH))
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
    fn traduz_a_janela_de_autenticacao() {
        let english = Catalog::new(&Locale::parse("en-US"), PORTUGUESE, ENGLISH).unwrap();
        assert_eq!(
            english.text("title").as_deref(),
            Some("System authentication")
        );
        assert_eq!(english.text("cancel").as_deref(), Some("Cancel"));
    }
}
