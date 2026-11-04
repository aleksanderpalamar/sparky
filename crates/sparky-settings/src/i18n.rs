use std::sync::OnceLock;

use fluent_bundle::FluentArgs;
use sparky_i18n::Catalog;

const PORTUGUESE: &str = include_str!("../../../data/i18n/pt-BR/settings.ftl");
const ENGLISH: &str = include_str!("../../../data/i18n/en/settings.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

pub fn text(key: &str) -> String {
    message(key, None)
}

pub fn message(key: &str, args: Option<&FluentArgs<'_>>) -> String {
    CATALOG
        .get_or_init(|| Catalog::for_domain("settings", PORTUGUESE, ENGLISH))
        .as_ref()
        .ok()
        .and_then(|catalog| catalog.message(key, args))
        .unwrap_or_else(|| key.to_owned())
}

pub fn with(key: &str, name: &str, value: &str) -> String {
    let mut args = FluentArgs::new();
    args.set(name, value);
    message(key, Some(&args))
}

#[cfg(test)]
mod tests {
    use sparky_i18n::{Catalog, Locale};

    use super::{ENGLISH, PORTUGUESE};

    #[test]
    fn traduz_navegacao_das_configuracoes() {
        let english = Catalog::new(&Locale::parse("en-US"), PORTUGUESE, ENGLISH).unwrap();
        assert_eq!(
            english.text("section-network-label").as_deref(),
            Some("Network")
        );
        assert_eq!(
            english.text("section-about-summary").as_deref(),
            Some("The system and Sparky")
        );
    }
}
