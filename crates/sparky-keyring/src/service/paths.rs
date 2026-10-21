use zbus::zvariant::{ObjectPath, OwnedObjectPath};

pub const SERVICE: &str = "/org/freedesktop/secrets";
pub const COLLECTION: &str = "/org/freedesktop/secrets/collection/login";
pub const DEFAULT_ALIAS: &str = "/org/freedesktop/secrets/aliases/default";
const NONE: &str = "/";
const SESSION: &str = "/org/freedesktop/secrets/session";
const PROMPT: &str = "/org/freedesktop/secrets/prompt";

pub fn none() -> OwnedObjectPath {
    ObjectPath::from_static_str_unchecked(NONE).into()
}

pub fn collection() -> OwnedObjectPath {
    ObjectPath::from_static_str_unchecked(COLLECTION).into()
}

pub fn item(id: u64) -> OwnedObjectPath {
    numbered(COLLECTION, id)
}

pub fn items(ids: &[u64]) -> Vec<OwnedObjectPath> {
    ids.iter().map(|id| item(*id)).collect()
}

pub fn session(number: u64) -> OwnedObjectPath {
    numbered(SESSION, number)
}

pub fn prompt(number: u64) -> OwnedObjectPath {
    numbered(PROMPT, number)
}

pub fn item_id(path: &str) -> Option<u64> {
    let rest = path.strip_prefix(COLLECTION).or_else(|| path.strip_prefix(DEFAULT_ALIAS))?;
    let number = rest.strip_prefix('/')?;
    match number.bytes().all(|byte| byte.is_ascii_digit()) {
        true => number.parse().ok(),
        false => None,
    }
}

pub fn is_collection(path: &str) -> bool {
    path == COLLECTION || path == DEFAULT_ALIAS
}

pub fn alias(name: &str) -> OwnedObjectPath {
    match name {
        "default" | "login" => collection(),
        _ => none(),
    }
}

fn numbered(prefix: &str, number: u64) -> OwnedObjectPath {
    ObjectPath::from_string_unchecked(format!("{prefix}/{number}")).into()
}

#[cfg(test)]
mod tests {
    use super::{COLLECTION, alias, is_collection, item, item_id, prompt, session};

    #[test]
    fn os_caminhos_seguem_o_padrao_do_secret_service() {
        assert_eq!(item(7).as_str(), "/org/freedesktop/secrets/collection/login/7");
        assert_eq!(session(2).as_str(), "/org/freedesktop/secrets/session/2");
        assert_eq!(prompt(3).as_str(), "/org/freedesktop/secrets/prompt/3");
    }

    #[test]
    fn o_numero_do_item_sai_do_caminho() {
        assert_eq!(item_id("/org/freedesktop/secrets/collection/login/7"), Some(7));
        assert_eq!(item_id("/org/freedesktop/secrets/aliases/default/8"), Some(8));
        assert_eq!(item_id(COLLECTION), None);
        assert_eq!(item_id("/org/freedesktop/secrets/collection/login/+7"), None);
        assert_eq!(item_id("/org/freedesktop/secrets/collection/outro/7"), None);
    }

    #[test]
    fn so_o_apelido_padrao_existe() {
        assert_eq!(alias("default").as_str(), COLLECTION);
        assert_eq!(alias("login").as_str(), COLLECTION);
        assert_eq!(alias("session").as_str(), "/");
        assert!(is_collection("/org/freedesktop/secrets/aliases/default"));
        assert!(!is_collection("/org/freedesktop/secrets/aliases/session"));
    }
}
