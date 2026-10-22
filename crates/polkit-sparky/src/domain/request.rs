use std::fmt;

use serde::{Deserialize, Serialize};

use super::Secret;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub uid: u32,
    pub name: String,
}

impl fmt::Display for Identity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.name, self.uid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub action_id: String,
    pub message: String,
    pub cookie: Secret,
    pub identities: Vec<Identity>,
}

impl Request {
    pub fn validate(&self) -> Result<(), String> {
        let cookie = self.cookie.expose();
        if cookie.is_empty() || cookie.len() > 4096 || cookie.contains(['\0', '\n', '\r']) {
            return Err("Pedido de autenticação inválido".to_owned());
        }
        if self.action_id.is_empty() || self.action_id.len() > 256 || self.message.len() > 8192 {
            return Err("Descrição de autenticação inválida".to_owned());
        }
        if self.identities.is_empty() || self.identities.len() > 64 {
            return Err("Nenhum usuário disponível para autenticar".to_owned());
        }
        if self
            .identities
            .iter()
            .any(|identity| identity.uid > i32::MAX as u32 || identity.name.len() > 256)
        {
            return Err("Identidade de autenticação inválida".to_owned());
        }
        Ok(())
    }
}
