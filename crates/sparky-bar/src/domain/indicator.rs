#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    #[default]
    Application,
    Network,
    Bluetooth,
    Audio,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    #[default]
    Active,
    Passive,
    Attention,
}

pub fn role(id: &str, icon: &str) -> Role {
    let id = id.to_lowercase();
    if id == "pasystray" || icon.starts_with("audio-volume-") {
        return Role::Audio;
    }
    if id.starts_with("blueman") || icon.starts_with("bluetooth") {
        return Role::Bluetooth;
    }
    if id == "nm-applet" || icon.starts_with("nm-") || icon.starts_with("network-") {
        return Role::Network;
    }
    Role::Application
}

pub fn icon_names(name: &str) -> Vec<String> {
    let preferred = match name {
        "nm-device-wired" | "network-wired" => "network-wired-symbolic",
        "nm-device-wired-autoip" => "network-wired-no-route-symbolic",
        "nm-device-wired-secure" => "network-wired-symbolic",
        "nm-no-connection" => "network-offline-symbolic",
        "nm-device-wireless" => "network-wireless-symbolic",
        "nm-device-wwan" => "network-cellular-symbolic",
        "nm-vpn-active-lock" | "nm-vpn-standalone-lock" => "network-vpn-symbolic",
        "bluetooth-active" | "bluetooth-paired" => "bluetooth-active-symbolic",
        "bluetooth-disabled" => "bluetooth-disabled-symbolic",
        name if name.starts_with("nm-signal-") => signal(name),
        name if name.starts_with("nm-stage") => "network-wireless-acquiring-symbolic",
        _ => "",
    };
    let mut names = Vec::new();
    if !preferred.is_empty() {
        names.push(preferred.to_owned());
    }
    if !name.ends_with("-symbolic") && !name.is_empty() {
        names.push(format!("{name}-symbolic"));
    }
    names.push(name.to_owned());
    names
}

fn signal(name: &str) -> &'static str {
    match name
        .trim_start_matches("nm-signal-")
        .trim_end_matches("-secure")
    {
        "100" => "network-wireless-signal-excellent-symbolic",
        "75" => "network-wireless-signal-good-symbolic",
        "50" => "network-wireless-signal-ok-symbolic",
        "25" => "network-wireless-signal-weak-symbolic",
        _ => "network-wireless-signal-none-symbolic",
    }
}
