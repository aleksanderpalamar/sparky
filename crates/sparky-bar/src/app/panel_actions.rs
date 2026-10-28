use iced::Task;
use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::app::{message::Message, panel_state::OpenPanel, state::Bar, tray_actions};
use crate::domain::{indicator::Role, panel::PanelKind};

pub fn kind(role: Role) -> Option<PanelKind> {
    match role {
        Role::Audio => Some(PanelKind::Audio),
        Role::Network => Some(PanelKind::Network),
        Role::Bluetooth => Some(PanelKind::Bluetooth),
        Role::Application => None,
    }
}

pub fn open(bar: &mut Bar, kind: PanelKind) -> Task<Message> {
    if bar.panels.open.is_some_and(|open| open.kind == kind) {
        return close(bar);
    }
    let previous = close(bar);
    let menu = tray_actions::close_menu(bar);
    let window = iced::window::Id::unique();
    bar.panels.open = Some(OpenPanel { window, kind });
    bar.panels.input_focus = super::panel_state::InputFocus::Interactive;
    bar.panels.errors.remove(&kind);
    let settings = NewLayerShellSettings {
        size: Some(if kind == PanelKind::Clock {
            (760, 520)
        } else {
            (400, 520)
        }),
        layer: Layer::Overlay,
        anchor: if kind == PanelKind::Clock {
            Anchor::Top
        } else {
            Anchor::Top | Anchor::Right
        },
        exclusive_zone: None,
        margin: Some((crate::ui::style::HEIGHT as i32 + 6, 10, 0, 0)),
        keyboard_interactivity: KeyboardInteractivity::Exclusive,
        output_option: OutputOption::LastOutput,
        events_transparent: false,
        namespace: Some("sparky-quick-panel".to_owned()),
    };
    let center = match kind {
        PanelKind::Clock => {
            bar.center.calendar.show_today();
            Task::batch([
                super::notifications::lease(bar, true),
                Task::perform(
                    async { tokio::task::spawn_blocking(super::center_geometry::query).await },
                    move |result| Message::CenterGeometry(window, result.unwrap_or((760, 520))),
                ),
            ])
        }
        _ => Task::none(),
    };
    Task::batch([
        previous,
        center,
        menu,
        Task::done(Message::NewLayerShell {
            settings,
            id: window,
        }),
    ])
}

pub fn yield_keyboard(bar: &mut Bar, kind: PanelKind) -> Task<Message> {
    let Some(open) = bar.panels.open.filter(|open| open.kind == kind) else {
        return Task::none();
    };
    bar.panels.input_focus = super::panel_state::InputFocus::Yielded;
    Task::done(Message::KeyboardInteractivityChange {
        id: open.window,
        keyboard_interactivity: KeyboardInteractivity::None,
    })
}

pub fn restore_keyboard(bar: &mut Bar, kind: PanelKind) -> Task<Message> {
    let Some(open) = bar.panels.open.filter(|open| open.kind == kind) else {
        return Task::none();
    };
    if bar.panels.input_focus == super::panel_state::InputFocus::Interactive {
        return Task::none();
    }
    bar.panels.input_focus = super::panel_state::InputFocus::Restoring;
    let focus = Task::done(Message::KeyboardInteractivityChange {
        id: open.window,
        keyboard_interactivity: KeyboardInteractivity::Exclusive,
    });
    let settled = Task::perform(
        async { tokio::time::sleep(std::time::Duration::from_millis(150)).await },
        move |_| Message::PanelFocusSettled(open.window),
    );
    Task::batch([focus, settled])
}

pub fn close(bar: &mut Bar) -> Task<Message> {
    let Some(open) = bar.panels.open.take() else {
        return Task::none();
    };
    if let Some(prompts) = &bar.panels.prompts {
        prompts.0.cancel(open.kind);
    }
    bar.panels.clear_input();
    let cleanup = match (open.kind, bar.panels.bluetooth_client.clone()) {
        (PanelKind::Bluetooth, Some(client)) => Task::perform(
            async move {
                client
                    .0
                    .send(crate::domain::bluetooth::BluetoothRequest::Cancel)
                    .await
            },
            |_| Message::Ignore,
        ),
        _ => Task::none(),
    };
    let lease = match open.kind {
        PanelKind::Clock => super::notifications::lease(bar, false),
        _ => Task::none(),
    };
    Task::batch([
        cleanup,
        lease,
        Task::done(Message::RemoveWindow(open.window)),
    ])
}
