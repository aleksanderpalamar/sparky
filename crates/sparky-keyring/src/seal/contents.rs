use std::collections::BTreeMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::domain::item::Item;
use crate::domain::secret::Secret;
use crate::domain::vault::Vault;

#[derive(Serialize, Deserialize)]
struct VaultData {
    created: u64,
    modified: u64,
    items: Vec<ItemData>,
}

#[derive(Serialize, Deserialize)]
struct ItemData {
    id: u64,
    label: String,
    attributes: BTreeMap<String, String>,
    secret: String,
    content_type: String,
    created: u64,
    modified: u64,
}

impl Drop for ItemData {
    fn drop(&mut self) {
        self.secret.zeroize();
    }
}

pub fn encode(vault: &Vault) -> Option<Zeroizing<Vec<u8>>> {
    let data = VaultData {
        created: vault.created,
        modified: vault.modified,
        items: vault.items().iter().map(item_data).collect(),
    };
    serde_json::to_vec(&data).ok().map(Zeroizing::new)
}

pub fn decode(bytes: &[u8]) -> Option<Vault> {
    let data: VaultData = serde_json::from_slice(bytes).ok()?;
    let items = data.items.iter().map(item_of).collect::<Option<Vec<_>>>()?;
    Some(Vault::restore(data.created, data.modified, items))
}

fn item_data(item: &Item) -> ItemData {
    ItemData {
        id: item.id,
        label: item.label.clone(),
        attributes: item.attributes.clone(),
        secret: BASE64.encode(item.secret.value.as_slice()),
        content_type: item.secret.content_type.clone(),
        created: item.created,
        modified: item.modified,
    }
}

fn item_of(data: &ItemData) -> Option<Item> {
    let value = BASE64.decode(&data.secret).ok()?;
    Some(Item {
        id: data.id,
        label: data.label.clone(),
        attributes: data.attributes.clone(),
        secret: Secret::new(value, data.content_type.clone()),
        created: data.created,
        modified: data.modified,
    })
}
