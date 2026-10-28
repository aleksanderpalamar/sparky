use super::bluetooth::ADDRESS;
use super::indicator::{Role, Status};
use super::tray::TrayItem;

pub struct Groups<'a> {
    pub pinned: Vec<&'a TrayItem>,
    pub drawer: Vec<&'a TrayItem>,
}

pub fn groups(items: &[TrayItem]) -> Groups<'_> {
    let external_bluetooth = items.iter().any(|item| {
        item.role == Role::Bluetooth && item.address != ADDRESS && item.status != Status::Passive
    });
    let native_audio = items.iter().any(|item| item.address == "sparky.audio");
    let native_network = items
        .iter()
        .any(|item| item.address == super::network::ADDRESS);
    let visible = items.iter().filter(|item| {
        if item.status == Status::Passive {
            return false;
        }
        match item.role {
            Role::Bluetooth => item.address != ADDRESS || !external_bluetooth,
            Role::Audio => !native_audio || item.address == "sparky.audio",
            Role::Network => !native_network || item.address == super::network::ADDRESS,
            Role::Application => true,
        }
    });
    let (pinned, drawer) = visible
        .partition(|item| item.role != Role::Application || item.status == Status::Attention);
    Groups { pinned, drawer }
}
