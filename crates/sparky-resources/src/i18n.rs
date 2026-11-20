use std::sync::OnceLock;

use fluent_bundle::FluentArgs;
use sparky_i18n::Catalog;

const PT: &str = include_str!("../../../data/i18n/pt-BR/resources.ftl");
const EN: &str = include_str!("../../../data/i18n/en/resources.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

#[cfg(test)]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::new(&sparky_i18n::Locale::parse("pt-BR"), PT, EN)
}

#[cfg(not(test))]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::for_domain("resources", PT, EN)
}

pub fn text(key: &str) -> String {
    message(key, &[])
}

pub fn message(key: &str, values: &[(&str, String)]) -> String {
    let mut args = FluentArgs::new();
    for (name, value) in values {
        args.set(*name, value.clone());
    }
    CATALOG
        .get_or_init(load_catalog)
        .as_ref()
        .ok()
        .and_then(|catalog| catalog.message(key, Some(&args)))
        .unwrap_or_else(|| key.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{EN, PT};
    use sparky_i18n::{Catalog, Locale};

    #[test]
    fn traduz_os_avisos_nos_dois_idiomas() {
        let english = Catalog::new(&Locale::parse("en-US"), PT, EN).ok();
        assert_eq!(
            english.and_then(|catalog| catalog.text("notice-reverted-title")),
            Some("Adjustment reverted".to_owned())
        );
    }
}
