use crate::Volume;

pub(super) fn phone(
    serial: Option<&str>,
    bus: Option<&str>,
    address: Option<&str>,
    label: Option<&str>,
) -> Option<Volume> {
    let host = serial
        .filter(|serial| !serial.is_empty())
        .map(escape)
        .or_else(|| usb_host(bus?, address?))?;
    Some(Volume {
        path: format!("mtp://{host}/"),
        size: String::new(),
        label: label
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("Dispositivo MTP")
            .to_owned(),
        filesystem: "mtp".to_owned(),
        mountpoint: None,
        removable: true,
    })
}

fn usb_host(bus: &str, address: &str) -> Option<String> {
    let bus = bus.parse::<u16>().ok().filter(|value| *value > 0)?;
    let address = address
        .parse::<u8>()
        .ok()
        .filter(|value| (1..=127).contains(value))?;
    Some(format!("[usb:{bus:03},{address:03}]"))
}

fn escape(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                char::from(byte).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests;
