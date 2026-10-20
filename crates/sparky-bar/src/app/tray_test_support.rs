use std::sync::{Arc, Mutex};

use iced::futures::future::BoxFuture;
use iced::futures::{StreamExt, executor::block_on};
use iced::{Task, window};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Bar;
use crate::app::tray_state::{OpenMenu, TrayClient};
use crate::domain::tray::{MenuEntry, PrimaryAction, TrayChange, TrayIcon, TrayItem, TrayRequest};
use crate::ports::fake::FakeWorkspaces;
use crate::ports::tray::TrayCommands;

pub fn bar() -> Bar {
    Bar::new(Skin::default(), Arc::new(FakeWorkspaces::default()), chrono::Local::now().naive_local())
}

pub fn item(address: &str) -> TrayItem {
    TrayItem {
        address: address.to_owned(),
        title: "Aplicativo de teste".to_owned(),
        icon: TrayIcon::Missing,
        primary_action: PrimaryAction::Activate,
        menu_path: Some("/Menu".to_owned()),
        menu: vec![
            MenuEntry { id: 1, label: "Abrir".to_owned(), enabled: true, separator: false },
            MenuEntry { id: 2, label: "Indisponível".to_owned(), enabled: false, separator: false },
            MenuEntry { id: 3, label: String::new(), enabled: true, separator: true },
        ],
    }
}

pub fn opened(bar: &mut Bar, address: &str) -> window::Id {
    bar.tray_mut().apply(TrayChange::Added(item(address)));
    let window = window::Id::unique();
    bar.tray_mut().open_menu(OpenMenu { window, address: address.to_owned() });
    window
}

pub fn messages(task: Task<Message>) -> Vec<Message> {
    let Some(stream) = iced_runtime::task::into_stream(task) else {
        return Vec::new();
    };
    block_on(
        stream
            .map(|action| match action {
                iced_runtime::Action::Output(message) => message,
                other => panic!("efeito inesperado no teste: {other:?}"),
            })
            .collect(),
    )
}

pub fn removes(messages: &[Message], window: window::Id) -> bool {
    messages.iter().any(|message| matches!(message, Message::RemoveWindow(found) if *found == window))
}

pub fn notifier(item_is_menu: bool) -> system_tray::item::StatusNotifierItem {
    system_tray::item::StatusNotifierItem {
        id: "teste".to_owned(),
        category: system_tray::item::Category::ApplicationStatus,
        title: None,
        status: system_tray::item::Status::Active,
        window_id: 0,
        icon_theme_path: None,
        icon_name: None,
        icon_pixmap: None,
        overlay_icon_name: None,
        overlay_icon_pixmap: None,
        attention_icon_name: None,
        attention_icon_pixmap: None,
        attention_movie_name: None,
        tool_tip: None,
        item_is_menu,
        menu: Some("/Menu".to_owned()),
    }
}

#[derive(Debug)]
pub struct FakeTray {
    response: Result<(), String>,
    requests: Mutex<Vec<TrayRequest>>,
}

impl FakeTray {
    pub fn requests(&self) -> Vec<TrayRequest> {
        match self.requests.lock() {
            Ok(requests) => requests.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

impl TrayCommands for FakeTray {
    fn send(&self, request: TrayRequest) -> BoxFuture<'_, Result<(), String>> {
        match self.requests.lock() {
            Ok(mut requests) => requests.push(request),
            Err(poisoned) => poisoned.into_inner().push(request),
        }
        Box::pin(std::future::ready(self.response.clone()))
    }
}

pub fn connected(response: Result<(), String>) -> (Bar, Arc<FakeTray>) {
    let mut bar = bar();
    let client = Arc::new(FakeTray { response, requests: Mutex::default() });
    bar.tray_mut().connect(TrayClient(client.clone()));
    (bar, client)
}
