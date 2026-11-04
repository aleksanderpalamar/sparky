use std::sync::OnceLock;

use sparky_i18n::Catalog;

const PORTUGUESE: &str = include_str!("../../../data/i18n/pt-BR/launcher.ftl");
const ENGLISH: &str = include_str!("../../../data/i18n/en/launcher.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

pub fn text(key: &str) -> String {
    CATALOG
        .get_or_init(|| Catalog::for_domain("launcher", PORTUGUESE, ENGLISH))
        .as_ref()
        .ok()
        .and_then(|catalog| catalog.text(key))
        .unwrap_or_else(|| key.to_owned())
}

#[cfg(test)]
mod tests {
    use sparky_i18n::{Catalog, Locale};

    use super::{ENGLISH, PORTUGUESE};

    #[test]
    fn traduz_menu_de_sessao() {
        let english = Catalog::new(&Locale::parse("en-US"), PORTUGUESE, ENGLISH).unwrap();
        assert_eq!(english.text("lock-name").as_deref(), Some("Lock"));
        assert_eq!(
            english.text("no-apps").as_deref(),
            Some("No applications found")
        );
    }
}
