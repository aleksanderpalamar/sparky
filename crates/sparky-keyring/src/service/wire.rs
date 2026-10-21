use std::collections::HashMap;
use std::mem;

use serde::{Deserialize, Serialize};
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Type};
use zeroize::Zeroize;

use crate::domain::item::Attributes;
use crate::domain::secret::Secret;

pub const LABEL: &str = "org.freedesktop.Secret.Item.Label";
pub const ATTRIBUTES: &str = "org.freedesktop.Secret.Item.Attributes";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct WireSecret {
    pub session: OwnedObjectPath,
    pub parameters: Vec<u8>,
    pub value: Vec<u8>,
    pub content_type: String,
}

impl WireSecret {
    pub fn of(session: OwnedObjectPath, secret: &Secret) -> Self {
        Self { session, parameters: Vec::new(), value: secret.value.to_vec(), content_type: secret.content_type.clone() }
    }

    pub fn into_secret(mut self) -> Secret {
        Secret::new(mem::take(&mut self.value), mem::take(&mut self.content_type))
    }
}

impl Drop for WireSecret {
    fn drop(&mut self) {
        self.value.zeroize();
    }
}

pub fn label_of(properties: &HashMap<String, OwnedValue>) -> String {
    properties
        .get(LABEL)
        .and_then(|value| <&str>::try_from(&**value).ok())
        .map(str::to_owned)
        .unwrap_or_default()
}

pub fn attributes_of(properties: &HashMap<String, OwnedValue>) -> Attributes {
    let Some(value) = properties.get(ATTRIBUTES).and_then(|value| value.try_clone().ok()) else {
        return Attributes::new();
    };
    HashMap::<String, String>::try_from(value).map(|pairs| pairs.into_iter().collect()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use zbus::zvariant::{OwnedValue, Str};

    use super::{ATTRIBUTES, LABEL, attributes_of, label_of};

    #[test]
    fn rotulo_e_atributos_saem_das_propriedades() {
        let atributos: HashMap<String, String> = [("service".to_owned(), "git".to_owned())].into_iter().collect();
        let propriedades: HashMap<String, OwnedValue> = [
            (LABEL.to_owned(), OwnedValue::from(Str::from("Git"))),
            (ATTRIBUTES.to_owned(), OwnedValue::from(atributos)),
        ]
        .into_iter()
        .collect();
        assert_eq!(label_of(&propriedades), "Git");
        assert_eq!(attributes_of(&propriedades).get("service").map(String::as_str), Some("git"));
    }

    #[test]
    fn propriedades_ausentes_viram_valores_vazios() {
        let vazias = HashMap::new();
        assert_eq!(label_of(&vazias), "");
        assert!(attributes_of(&vazias).is_empty());
    }
}
