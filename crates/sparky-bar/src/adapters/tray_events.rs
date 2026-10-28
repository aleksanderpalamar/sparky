use system_tray::client::{Event, UpdateEvent};
use system_tray::item::{IconPixmap, Status as ItemStatus, StatusNotifierItem};
use system_tray::menu::{MenuItem, MenuType, TrayMenu};

use crate::adapters::icons::Icons;
use crate::domain::indicator::{Role, Status, role};
use crate::domain::label::menu_label;
use crate::domain::pixmap::{argb_to_rgba, best_size};
use crate::domain::tray::{MenuEntry, PrimaryAction, TrayChange, TrayIcon, TrayItem};

const ICON_SIZE: i32 = 22;

pub fn changes(
    event: Event,
    snapshot: impl FnOnce(&str) -> Option<TrayItem>,
    icons: &Icons,
) -> Vec<TrayChange> {
    match event {
        Event::Add(address, item) => vec![TrayChange::Added(item_of(address, *item, None, icons))],
        Event::Remove(address) => vec![TrayChange::Removed(address)],
        Event::Update(address, UpdateEvent::MenuDiff(_)) => snapshot(&address)
            .map(|item| TrayChange::Menu {
                address,
                menu: item.menu,
            })
            .into_iter()
            .collect(),
        Event::Update(address, update) => {
            let found = snapshot(&address)
                .map(TrayChange::Added)
                .or_else(|| update_of(address, update, icons));
            found.into_iter().collect()
        }
    }
}

pub fn item_of(
    address: String,
    item: StatusNotifierItem,
    menu: Option<&TrayMenu>,
    icons: &Icons,
) -> TrayItem {
    let role = role(&item.id, item.icon_name.as_deref().unwrap_or_default());
    let attention = item.status == ItemStatus::NeedsAttention;
    let name = match attention {
        true => item
            .attention_icon_name
            .as_deref()
            .or(item.icon_name.as_deref()),
        false => item.icon_name.as_deref(),
    };
    let pixmaps = match attention {
        true => item
            .attention_icon_pixmap
            .as_deref()
            .or(item.icon_pixmap.as_deref()),
        false => item.icon_pixmap.as_deref(),
    };
    let icon = icon_of(icons, name, item.icon_theme_path.as_deref(), pixmaps, role);
    TrayItem {
        title: item
            .title
            .filter(|title| !title.is_empty())
            .unwrap_or(item.id),
        icon,
        role,
        status: status(item.status),
        primary_action: match item.item_is_menu {
            true => PrimaryAction::Menu,
            false => PrimaryAction::Activate,
        },
        menu_path: item.menu,
        menu: menu.map(|menu| entries(&menu.submenus)).unwrap_or_default(),
        address,
    }
}

fn update_of(address: String, update: UpdateEvent, icons: &Icons) -> Option<TrayChange> {
    match update {
        UpdateEvent::Icon {
            icon_name,
            icon_pixmap,
        } => Some(TrayChange::Icon {
            icon: icon_of(
                icons,
                icon_name.as_deref(),
                None,
                icon_pixmap.as_deref(),
                Role::Application,
            ),
            address,
        }),
        UpdateEvent::Status(value) => Some(TrayChange::Status {
            address,
            status: status(value),
        }),
        UpdateEvent::Title(Some(title)) => Some(TrayChange::Title { address, title }),
        UpdateEvent::Menu(menu) => Some(TrayChange::Menu {
            menu: entries(&menu.submenus),
            address,
        }),
        _ => None,
    }
}

fn status(status: ItemStatus) -> Status {
    match status {
        ItemStatus::Passive => Status::Passive,
        ItemStatus::NeedsAttention => Status::Attention,
        _ => Status::Active,
    }
}

fn icon_of(
    icons: &Icons,
    name: Option<&str>,
    path: Option<&str>,
    pixmaps: Option<&[IconPixmap]>,
    role: Role,
) -> TrayIcon {
    let named = name
        .filter(|name| !name.is_empty())
        .and_then(|name| icons.resolve(name, path));
    let fallback = match role {
        Role::Network => Some("network-transmit-receive-symbolic"),
        Role::Bluetooth => Some("bluetooth-symbolic"),
        Role::Application => None,
    };
    named
        .or_else(|| fallback.and_then(|name| icons.resolve(name, None)))
        .or_else(|| pixmaps.and_then(pixels_of))
        .unwrap_or(TrayIcon::Missing)
}

fn pixels_of(pixmaps: &[IconPixmap]) -> Option<TrayIcon> {
    let sizes: Vec<(i32, i32)> = pixmaps
        .iter()
        .map(|pixmap| (pixmap.width, pixmap.height))
        .collect();
    let pixmap = pixmaps.get(best_size(&sizes, ICON_SIZE)?)?;
    Some(TrayIcon::Pixels {
        width: pixmap.width.max(0) as u32,
        height: pixmap.height.max(0) as u32,
        rgba: argb_to_rgba(&pixmap.pixels),
    })
}

fn entries(items: &[MenuItem]) -> Vec<MenuEntry> {
    items
        .iter()
        .filter(|item| item.visible)
        .map(|item| MenuEntry {
            id: item.id,
            label: menu_label(item.label.as_deref().unwrap_or_default()),
            enabled: item.enabled,
            separator: item.menu_type == MenuType::Separator,
        })
        .collect()
}
