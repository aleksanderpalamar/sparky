use std::sync::OnceLock;

use fluent_bundle::FluentArgs;
use sparky_i18n::Catalog;

const PORTUGUESE: &str = include_str!("../../../data/i18n/pt-BR/store.ftl");
const ENGLISH: &str = include_str!("../../../data/i18n/en/store.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

#[cfg(test)]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::new(&sparky_i18n::Locale::parse("pt-BR"), PORTUGUESE, ENGLISH)
}

#[cfg(not(test))]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::for_domain("store", PORTUGUESE, ENGLISH)
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
    use super::{ENGLISH, PORTUGUESE};
    use fluent_bundle::FluentArgs;
    use sparky_i18n::{Catalog, Locale};

    #[test]
    fn ambos_os_idiomas_cobrem_as_acoes() {
        for locale in ["en-US", "pt-BR"] {
            let catalog = Catalog::new(&Locale::parse(locale), PORTUGUESE, ENGLISH).unwrap();
            for key in [
                "title",
                "install",
                "update",
                "remove",
                "confirm",
                "flathub-unavailable",
                "update-all",
                "up-to-date",
                "flatpak-runtime",
            ] {
                assert!(catalog.text(key).is_some(), "{locale}: {key}");
            }
            let mut args = FluentArgs::new();
            args.set("count", 2);
            assert!(
                catalog
                    .message("confirm-all-updates", Some(&args))
                    .is_some()
            );
        }
    }
}
