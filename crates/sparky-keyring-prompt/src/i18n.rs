use std::sync::OnceLock;

use sparky_i18n::Catalog;

const PORTUGUESE: &str = include_str!("../../../data/i18n/pt-BR/keyring.ftl");
const ENGLISH: &str = include_str!("../../../data/i18n/en/keyring.ftl");
static CATALOG: OnceLock<Result<Catalog, String>> = OnceLock::new();

pub fn text(key: &str) -> String {
    CATALOG
        .get_or_init(|| Catalog::for_domain("keyring", PORTUGUESE, ENGLISH))
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
    fn traduz_o_pedido_de_senha_do_cofre() {
        let english = Catalog::new(&Locale::parse("en-US"), PORTUGUESE, ENGLISH).unwrap();
        assert_eq!(
            english.text("create-title").as_deref(),
            Some("Create a vault password")
        );
        assert_eq!(
            english.text("retry-warning").as_deref(),
            Some("Incorrect password. Try again.")
        );
    }
}
