use std::collections::{HashMap, HashSet};

use zbus::zvariant::{OwnedValue, Value};

use crate::domain::{Identity, Request, Secret};

pub type WireIdentity = (String, HashMap<String, OwnedValue>);

pub fn request(
    action: &str,
    message: &str,
    cookie: &str,
    identities: Vec<WireIdentity>,
    username: impl Fn(u32) -> String,
) -> Result<Request, String> {
    let mut users = Vec::new();
    let mut seen = HashSet::new();
    for (kind, details) in identities {
        if kind != "unix-user" {
            continue;
        }
        let Some(Value::U32(uid)) = details.get("uid").map(|value| &**value) else {
            continue;
        };
        if *uid > i32::MAX as u32 || !seen.insert(*uid) {
            continue;
        }
        users.push(Identity {
            uid: *uid,
            name: username(*uid),
        });
    }
    let request = Request {
        action_id: action.to_owned(),
        message: message.to_owned(),
        cookie: Secret::new(cookie.to_owned()),
        identities: users,
    };
    request.validate()?;
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::request;
    use zbus::zvariant::OwnedValue;

    #[test]
    fn ignora_grupos_e_usuarios_invalidos_e_elimina_repeticoes() {
        let identities = vec![
            (
                "unix-group".to_owned(),
                [("uid".to_owned(), OwnedValue::from(1u32))].into(),
            ),
            (
                "unix-user".to_owned(),
                [("uid".to_owned(), OwnedValue::from(1000u32))].into(),
            ),
            (
                "unix-user".to_owned(),
                [("uid".to_owned(), OwnedValue::from(1000u32))].into(),
            ),
        ];
        let request = request("ação", "Autentique", "fixture", identities, |uid| {
            format!("user-{uid}")
        })
        .unwrap();
        assert_eq!(request.identities.len(), 1);
        assert_eq!(request.identities[0].uid, 1000);
        assert!(
            super::request("ação", "Autentique", "fixture", Vec::new(), |_| {
                String::new()
            })
            .is_err()
        );
    }
}
