use fluent_bundle::FluentArgs;
use sparky_i18n::Catalog;
use std::sync::OnceLock;

const PT: &str = include_str!("../../../data/i18n/pt-BR/monitor.ftl");
const EN: &str = include_str!("../../../data/i18n/en/monitor.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

#[cfg(test)]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::new(&sparky_i18n::Locale::parse("pt-BR"), PT, EN)
}

#[cfg(not(test))]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::for_domain("monitor", PT, EN)
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

#[cfg(test)]
mod tests {
    use super::{EN, PT};
    use sparky_i18n::{Catalog, Locale};

    #[test]
    fn traduz_paginas_do_monitor() {
        let catalog = Catalog::new(&Locale::parse("en-US"), PT, EN).unwrap();
        assert_eq!(catalog.text("dashboard").as_deref(), Some("Dashboard"));
        assert_eq!(catalog.text("memory").as_deref(), Some("Memory"));
    }
}
