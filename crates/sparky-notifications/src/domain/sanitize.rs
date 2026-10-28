use super::{Picture, Urgency};

pub fn text(value: &str, limit: usize) -> String {
    let end = value
        .char_indices()
        .nth(limit)
        .map_or(value.len(), |(index, _)| index);
    value[..end]
        .chars()
        .filter(|value| !value.is_control() || matches!(value, '\n' | '\t'))
        .collect()
}

pub fn picture(picture: &Picture) -> bool {
    picture.width > 0
        && picture.height > 0
        && picture.width <= 64
        && picture.height <= 64
        && picture.rgba.len() == picture.width as usize * picture.height as usize * 4
}

pub fn timeout(value: i32, urgency: Urgency) -> Option<u64> {
    if urgency == Urgency::Critical {
        return None;
    }
    match value {
        0 => None,
        value if value > 0 => Some(value as u64),
        _ => Some(5000),
    }
}
