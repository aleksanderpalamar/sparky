use iced::Task;
use iced::futures::stream::BoxStream;
use iced::futures::{Stream, StreamExt};

use crate::adapters::bluetooth::Bluez;
use crate::adapters::icons::Icons;
use crate::app::message::Message;
use crate::app::state::Bar;
use crate::domain::bluetooth::{ADDRESS, Bluetooth};
use crate::domain::indicator::{Role, Status};
use crate::domain::tray::{PrimaryAction, TrayChange, TrayIcon, TrayItem};
use crate::ports::bluetooth::BluetoothSource;

pub fn events() -> impl Stream<Item = Message> {
    let icons = Icons::current();
    events_from(&Bluez, move |name| icons.resolve(name, None))
}

pub fn events_from(
    source: &impl BluetoothSource,
    resolve: impl Fn(&str) -> Option<TrayIcon> + Send + 'static,
) -> BoxStream<'static, Message> {
    source
        .subscribe()
        .map(move |state| Message::Bluetooth(state.map(|state| item(state, &resolve))))
        .boxed()
}

pub fn apply(bar: &mut Bar, item: Option<TrayItem>) -> Task<Message> {
    let change = match item {
        Some(item) => TrayChange::Added(item),
        None => TrayChange::Removed(ADDRESS.to_owned()),
    };
    super::tray_actions::apply(bar, change)
}

fn item(bluetooth: Bluetooth, resolve: &impl Fn(&str) -> Option<TrayIcon>) -> TrayItem {
    TrayItem {
        address: ADDRESS.to_owned(),
        title: bluetooth.title().to_owned(),
        icon: resolve(bluetooth.icon()).unwrap_or(TrayIcon::Missing),
        role: Role::Bluetooth,
        status: Status::Active,
        primary_action: PrimaryAction::Menu,
        menu_path: None,
        menu: bluetooth.menu(),
    }
}
