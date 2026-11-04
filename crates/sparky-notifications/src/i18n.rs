use std::sync::OnceLock;

use sparky_i18n::Catalog;

const PT: &str = include_str!("../../../data/i18n/pt-BR/notifications.ftl");
const EN: &str = include_str!("../../../data/i18n/en/notifications.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

pub fn text(key: &str) -> String {
    CATALOG
        .get_or_init(|| Catalog::for_domain("notifications", PT, EN))
        .as_ref()
        .ok()
        .and_then(|catalog| catalog.text(key))
        .unwrap_or_else(|| key.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{EN, PT};
    use sparky_i18n::{Catalog, Locale};

    #[test]
    fn traduz_erros_do_servico() {
        let english = Catalog::new(&Locale::parse("en-US"), PT, EN).unwrap();
        assert_eq!(
            english.text("active-limit").as_deref(),
            Some("Active notification limit reached")
        );
    }
}
