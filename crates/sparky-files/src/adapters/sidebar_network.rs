use gio::prelude::*;

use crate::domain::sidebar_location::{SidebarItem, SidebarTarget};

pub(super) fn item(root: &gio::File, info: &gio::FileInfo) -> SidebarItem {
    let child = root.child(info.name());
    let target = match info.file_type() {
        gio::FileType::Shortcut => info
            .attribute_string("standard::target-uri")
            .filter(|uri| !uri.is_empty()),
        _ => None,
    };
    SidebarItem {
        target: SidebarTarget::Remote {
            uri: target.unwrap_or_else(|| child.uri()).to_string(),
            directory: info.file_type() != gio::FileType::Regular,
        },
        name: info.display_name().to_string(),
    }
}
