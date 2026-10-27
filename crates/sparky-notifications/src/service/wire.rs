use crate::domain::{Action, Error, Incoming, Urgency};
use std::collections::HashMap;
use zbus::{fdo, message::Header, zvariant::OwnedValue};

pub fn incoming(
    mut input: Incoming,
    actions: Vec<String>,
    hints: &HashMap<String, OwnedValue>,
) -> Incoming {
    input.urgency = match hints
        .get("urgency")
        .and_then(|value| u8::try_from(value).ok())
    {
        Some(0) => Urgency::Low,
        Some(2) => Urgency::Critical,
        _ => Urgency::Normal,
    };
    input.actions = actions
        .chunks_exact(2)
        .map(|pair| Action {
            key: pair[0].clone(),
            label: pair[1].clone(),
        })
        .collect();
    if input.icon.is_empty() {
        input.icon = hints
            .get("desktop-entry")
            .and_then(|value| <&str>::try_from(value).ok())
            .unwrap_or("")
            .to_owned();
    }
    input.transient = flag(hints, "transient");
    input.resident = flag(hints, "resident");
    input
}

pub fn sender(header: &Header<'_>) -> fdo::Result<String> {
    header
        .sender()
        .map(|sender| sender.to_string())
        .ok_or_else(|| fdo::Error::AccessDenied("Solicitante indisponível".to_owned()))
}

pub fn error(error: Error) -> fdo::Error {
    match error {
        Error::Sender => fdo::Error::AccessDenied(error.to_string()),
        Error::Unknown => fdo::Error::UnknownObject(error.to_string()),
        Error::Full => fdo::Error::LimitsExceeded(error.to_string()),
        Error::Poisoned => fdo::Error::Failed(error.to_string()),
    }
}

fn flag(hints: &HashMap<String, OwnedValue>, key: &str) -> bool {
    hints
        .get(key)
        .and_then(|value| bool::try_from(value).ok())
        .unwrap_or(false)
}
