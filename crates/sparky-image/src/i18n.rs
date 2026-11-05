use std::sync::OnceLock;

use fluent_bundle::FluentArgs;
use sparky_i18n::Catalog;

const PT: &str = include_str!("../../../data/i18n/pt-BR/image.ftl");
const EN: &str = include_str!("../../../data/i18n/en/image.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

#[cfg(test)]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::new(&sparky_i18n::Locale::parse("pt-BR"), PT, EN)
}

#[cfg(not(test))]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::for_domain("image", PT, EN)
}

pub fn text(key: &str) -> String {
    message(key, None)
}

pub fn message(key: &str, args: Option<&FluentArgs<'_>>) -> String {
    CATALOG
        .get_or_init(load_catalog)
        .as_ref()
        .ok()
        .and_then(|catalog| catalog.message(key, args))
        .unwrap_or_else(|| key.to_owned())
}

pub fn with(key: &str, name: &str, value: String) -> String {
    let mut args = FluentArgs::new();
    args.set(name, value);
    message(key, Some(&args))
}

#[cfg(test)]
mod tests {
    use super::{EN, PT};
    use sparky_i18n::{Catalog, Locale};

    #[test]
    fn traduz_mensagens_do_visualizador() {
        let catalog = Catalog::new(&Locale::parse("en-US"), PT, EN).unwrap();
        assert_eq!(catalog.text("loading").as_deref(), Some("Loading…"));
        assert_eq!(
            catalog.text("unsupported").as_deref(),
            Some("Unsupported image format")
        );
    }
}
