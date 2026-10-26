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
    let visible = items.iter().filter(|item| {
        item.status != Status::Passive && (item.address != ADDRESS || !external_bluetooth)
    });
    let (pinned, drawer) = visible
        .partition(|item| item.role != Role::Application || item.status == Status::Attention);
    Groups { pinned, drawer }
}
