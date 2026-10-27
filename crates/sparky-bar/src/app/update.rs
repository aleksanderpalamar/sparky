use iced::Task;

use crate::app::message::Message;
use crate::app::state::Bar;
use crate::app::tray_actions;

pub fn update(bar: &mut Bar, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => bar.apply_palette(palette),
        Message::WorkspacesChanged => bar.refresh_workspaces(),
        Message::Tick => bar.tick(chrono::Local::now().naive_local()),
        Message::FocusWorkspace(id) => report(bar.focus(id)),
        Message::ToggleClock => bar.toggle_clock(),
        Message::Notifications(event) => return super::notifications::event(bar, event),
        Message::NotificationCommand(command) => {
            return super::notifications::request(bar, command);
        }
        Message::NotificationFinished(id, result) => {
            bar.panels
                .finish(id, crate::domain::panel::PanelKind::Clock, result)
        }
        Message::Calendar(action) => bar.center.calendar.apply(action),
        Message::CenterGeometry(id, size) => return super::center_geometry::apply(bar, id, size),
        Message::Tray(change) => return tray_actions::apply(bar, change),
        Message::Bluetooth(item, state) => {
            bar.panels.bluetooth = state;
            return super::bluetooth::apply(bar, item);
        }
        Message::TrayHover(window, expansion) => {
            let open = bar.tray().menu().is_some();
            bar.tray_mut()
                .drawers
                .hover(window, expansion, open, std::time::Instant::now());
        }
        Message::TrayFrame(now) => bar.tray_mut().drawers.advance(now),
        Message::TrayReady(client) => bar.tray_mut().connect(client),
        Message::TrayActivate(address) => return tray_actions::activate(bar, address),
        Message::TrayMenu(address) => return tray_actions::open_menu(bar, address),
        Message::TrayChoose(entry) => return tray_actions::choose(bar, entry),
        Message::CloseMenu => {
            return Task::batch([
                super::panel_actions::close(bar),
                tray_actions::close_menu(bar),
            ]);
        }
        other => return super::panel_update::update(bar, other),
    }
    Task::none()
}

fn report(result: Result<(), sparky_ipc::IpcError>) {
    if let Err(error) = result {
        eprintln!("sparky-bar: o sparky recusou trocar de área: {error}");
    }
}
