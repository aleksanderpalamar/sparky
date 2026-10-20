use iced::Task;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption};
use system_tray::client::ActivateRequest;

use crate::app::message::Message;
use crate::app::state::Bar;
use crate::app::tray_state::OpenMenu;
use crate::ui::style::{HEIGHT, MENU_ROW, MENU_WIDTH};

pub fn activate(bar: &Bar, address: String) -> Task<Message> {
    let has_menu = bar.tray().tray().item(&address).is_some_and(|item| !item.menu.is_empty());
    let Some(client) = bar.tray().client() else {
        return Task::none();
    };
    let request = ActivateRequest::Default { address: address.clone(), x: 0, y: 0 };
    Task::perform(async move { client.0.activate(request).await.is_ok() }, move |accepted| {
        match (accepted, has_menu) {
            (false, true) => Message::TrayMenu(address.clone()),
            _ => Message::Ignore,
        }
    })
}

pub fn open_menu(bar: &mut Bar, address: String) -> Task<Message> {
    let Some(item) = bar.tray().tray().item(&address) else {
        return Task::none();
    };
    if item.menu.is_empty() {
        return Task::none();
    }
    let height = (item.menu.len() as u32 * MENU_ROW as u32) + 12;
    let window = iced::window::Id::unique();
    let settings = NewLayerShellSettings {
        size: Some((MENU_WIDTH as u32, height)),
        layer: Layer::Overlay,
        anchor: Anchor::Top | Anchor::Right,
        exclusive_zone: None,
        margin: Some((HEIGHT as i32, 6, 0, 0)),
        keyboard_interactivity: KeyboardInteractivity::OnDemand,
        output_option: OutputOption::LastOutput,
        events_transparent: false,
        namespace: Some("sparky-bar-menu".to_owned()),
    };
    let open = Task::done(Message::NewLayerShell { settings, id: window });
    let previous = bar.tray_mut().open_menu(OpenMenu { window, address });
    Task::batch([close(previous), open])
}

pub fn choose(bar: &mut Bar, entry: i32) -> Task<Message> {
    let menu = bar.tray_mut().close_menu();
    let Some(menu) = menu else {
        return Task::none();
    };
    let target = bar.tray().tray().item(&menu.address).and_then(|item| item.menu_path.clone());
    let activation = match (bar.tray().client(), target) {
        (Some(client), Some(menu_path)) => {
            let request = ActivateRequest::MenuItem { address: menu.address.clone(), menu_path, submenu_id: entry };
            Task::perform(async move { client.0.activate(request).await.is_ok() }, |_| Message::Ignore)
        }
        _ => Task::none(),
    };
    Task::batch([close(Some(menu)), activation])
}

pub fn close_menu(bar: &mut Bar) -> Task<Message> {
    close(bar.tray_mut().close_menu())
}

fn close(menu: Option<OpenMenu>) -> Task<Message> {
    match menu {
        Some(menu) => Task::done(Message::RemoveWindow(menu.window)),
        None => Task::none(),
    }
}
