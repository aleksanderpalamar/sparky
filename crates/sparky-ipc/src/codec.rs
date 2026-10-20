use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::IpcError;

pub fn encode<T: Serialize>(message: &T) -> Result<String, IpcError> {
    let mut line = serde_json::to_string(message).map_err(|error| IpcError::Malformed(error.to_string()))?;
    line.push('\n');
    Ok(line)
}

pub fn decode<T: DeserializeOwned>(line: &str) -> Result<T, IpcError> {
    serde_json::from_str(line.trim()).map_err(|error| IpcError::Malformed(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};
    use crate::error::IpcError;
    use crate::message::Request;

    #[test]
    fn cada_mensagem_e_uma_linha() {
        assert_eq!(encode(&Request::Workspaces).ok().as_deref(), Some("{\"request\":\"workspaces\"}\n"));
    }

    #[test]
    fn le_a_linha_com_ou_sem_quebra() {
        assert_eq!(decode::<Request>("{\"request\":\"subscribe\"}\n"), Ok(Request::Subscribe));
    }

    #[test]
    fn linha_invalida_e_mensagem_malformada() {
        assert!(matches!(decode::<Request>("oi"), Err(IpcError::Malformed(_))));
        assert!(matches!(decode::<Request>("{\"request\":\"voar\"}"), Err(IpcError::Malformed(_))));
    }
}
