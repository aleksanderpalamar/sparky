use std::sync::OnceLock;

use fluent_bundle::FluentArgs;
use sparky_i18n::Catalog;

const PT: &str = include_str!("../../../data/i18n/pt-BR/shot.ftl");
const EN: &str = include_str!("../../../data/i18n/en/shot.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

#[cfg(test)]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::new(&sparky_i18n::Locale::parse("pt-BR"), PT, EN)
}

#[cfg(not(test))]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::for_domain("shot", PT, EN)
}

pub fn text(key: &str) -> String {
    CATALOG
        .get_or_init(load_catalog)
        .as_ref()
        .ok()
        .and_then(|catalog| catalog.text(key))
        .unwrap_or_else(|| key.to_owned())
}

pub fn with(key: &str, name: &str, value: String) -> String {
    let mut args = FluentArgs::new();
    args.set(name, value);
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
    fn traduz_seletor_e_compartilhamento() {
        let catalog = Catalog::new(&Locale::parse("en-US"), PT, EN).unwrap();
        assert_eq!(
            catalog.text("capture-title").as_deref(),
            Some("Capture screen")
        );
        assert_eq!(catalog.text("share-title").as_deref(), Some("Share screen"));
    }
}
