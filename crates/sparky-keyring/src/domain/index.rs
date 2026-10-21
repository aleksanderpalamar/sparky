use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::domain::item::Attributes;
use crate::domain::vault::Vault;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub id: u64,
    pub attributes: Vec<String>,
}

pub fn fingerprint(name: &str, value: &str) -> String {
    let digest = Sha256::new().chain_update(name).chain_update([0]).chain_update(value).finalize();
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn index_of(vault: &Vault) -> Vec<Entry> {
    vault
        .items()
        .iter()
        .map(|item| Entry { id: item.id, attributes: fingerprints(&item.attributes) })
        .collect()
}

pub fn search(index: &[Entry], query: &Attributes) -> Vec<u64> {
    let wanted = fingerprints(query);
    index
        .iter()
        .filter(|entry| wanted.iter().all(|print| entry.attributes.contains(print)))
        .map(|entry| entry.id)
        .collect()
}

fn fingerprints(attributes: &Attributes) -> Vec<String> {
    attributes.iter().map(|(name, value)| fingerprint(name, value)).collect()
}
