use system_tray::client::{Event, UpdateEvent};
use system_tray::item::{IconPixmap, StatusNotifierItem};
use system_tray::menu::{MenuItem, MenuType, TrayMenu};

use crate::adapters::icons;
use crate::domain::label::menu_label;
use crate::domain::pixmap::{argb_to_rgba, best_size};
use crate::domain::tray::{MenuEntry, PrimaryAction, TrayChange, TrayIcon, TrayItem};

const ICON_SIZE: i32 = 22;

pub fn changes(event: Event, snapshot: impl FnOnce(&str) -> Option<TrayItem>) -> Vec<TrayChange> {
    match event {
        Event::Add(address, item) => vec![TrayChange::Added(item_of(address, *item, None))],
        Event::Remove(address) => vec![TrayChange::Removed(address)],
        Event::Update(address, UpdateEvent::MenuDiff(_)) => {
            snapshot(&address).map(|item| TrayChange::Menu { address, menu: item.menu }).into_iter().collect()
        }
        Event::Update(address, update) => update_of(address, update).into_iter().collect(),
    }
}

pub fn item_of(address: String, item: StatusNotifierItem, menu: Option<&TrayMenu>) -> TrayItem {
    let icon = icon_of(item.icon_name.as_deref(), item.icon_theme_path.as_deref(), item.icon_pixmap.as_deref());
    TrayItem {
        title: item.title.clone().filter(|title| !title.is_empty()).unwrap_or(item.id),
        icon,
        primary_action: match item.item_is_menu {
            true => PrimaryAction::Menu,
            false => PrimaryAction::Activate,
        },
        menu_path: item.menu,
        menu: menu.map(|menu| entries(&menu.submenus)).unwrap_or_default(),
        address,
    }
}

fn update_of(address: String, update: UpdateEvent) -> Option<TrayChange> {
    match update {
        UpdateEvent::Icon { icon_name, icon_pixmap } => {
            Some(TrayChange::Icon { icon: icon_of(icon_name.as_deref(), None, icon_pixmap.as_deref()), address })
        }
        UpdateEvent::Title(Some(title)) => Some(TrayChange::Title { address, title }),
        UpdateEvent::Menu(menu) => Some(TrayChange::Menu { menu: entries(&menu.submenus), address }),
        _ => None,
    }
}

fn icon_of(name: Option<&str>, theme_path: Option<&str>, pixmaps: Option<&[IconPixmap]>) -> TrayIcon {
    let named =
        name.filter(|name| !name.is_empty()).and_then(|name| icons::resolve(name, theme_path)).map(TrayIcon::File);
    named.or_else(|| pixmaps.and_then(pixels_of)).unwrap_or(TrayIcon::Missing)
}

fn pixels_of(pixmaps: &[IconPixmap]) -> Option<TrayIcon> {
    let sizes: Vec<(i32, i32)> = pixmaps.iter().map(|pixmap| (pixmap.width, pixmap.height)).collect();
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
