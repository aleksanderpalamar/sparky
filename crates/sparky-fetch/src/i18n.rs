use std::sync::OnceLock;

use fluent_bundle::FluentArgs;
use sparky_i18n::Catalog;

const PT: &str = include_str!("../../../data/i18n/pt-BR/fetch.ftl");
const EN: &str = include_str!("../../../data/i18n/en/fetch.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

#[cfg(test)]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::new(&sparky_i18n::Locale::parse("pt-BR"), PT, EN)
}

#[cfg(not(test))]
fn load_catalog() -> Result<Catalog, String> {
    Catalog::for_domain("fetch", PT, EN)
}

pub fn text(key: &str) -> String {
    message(key, None)
}

pub fn with(key: &str, name: &str, value: &str) -> String {
    pairs(key, &[(name, value)])
}

pub fn pairs(key: &str, values: &[(&str, &str)]) -> String {
    let mut args = FluentArgs::new();
    for (name, value) in values {
        args.set(*name, *value);
    }
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
    use super::{EN, PT};
    use crate::domain::field::Field;
    use sparky_i18n::{Catalog, Locale};

    #[test]
    fn todo_campo_tem_rotulo_nos_dois_idiomas() {
        for locale in ["pt-BR", "en-US"] {
            let catalog = Catalog::new(&Locale::parse(locale), PT, EN)
                .unwrap_or_else(|error| panic!("{error}"));
            for field in Field::ALL {
                let key = format!("field-{}", field.key());
                assert!(catalog.text(&key).is_some(), "{locale}: {key}");
            }
        }
    }
}
