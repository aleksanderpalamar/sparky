use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const RPC: u64 = 1;

pub fn answer(password: &str, salt: &str, challenge: &str) -> String {
    let secret = STANDARD.encode(Sha256::digest(format!("{password}{salt}")));
    STANDARD.encode(Sha256::digest(format!("{secret}{challenge}")))
}

pub fn challenge(hello: &str) -> Option<Option<(String, String)>> {
    let message: Value = serde_json::from_str(hello).ok()?;
    if message["op"].as_u64() != Some(0) {
        return None;
    }
    let auth = &message["d"]["authentication"];
    Some(
        auth["salt"]
            .as_str()
            .zip(auth["challenge"].as_str())
            .map(|(salt, challenge)| (salt.to_owned(), challenge.to_owned())),
    )
}

pub fn identify(authentication: Option<String>) -> String {
    let mut data = json!({"rpcVersion": RPC, "eventSubscriptions": 0});
    if let Some(authentication) = authentication {
        data["authentication"] = Value::from(authentication);
    }
    json!({"op": 1, "d": data}).to_string()
}

pub fn identified(text: &str) -> bool {
    serde_json::from_str::<Value>(text).is_ok_and(|message| message["op"].as_u64() == Some(2))
}

pub fn request(kind: &str) -> String {
    json!({"op": 6, "d": {"requestType": kind, "requestId": kind}}).to_string()
}

pub fn active(text: &str, kind: &str) -> Option<bool> {
    let message: Value = serde_json::from_str(text).ok()?;
    let data = &message["d"];
    (message["op"].as_u64() == Some(7) && data["requestId"].as_str() == Some(kind))
        .then(|| data["responseData"]["outputActive"].as_bool())
        .flatten()
}
