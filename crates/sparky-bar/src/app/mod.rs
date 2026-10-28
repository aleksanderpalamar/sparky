pub mod audio;
pub mod bluetooth;
pub mod drawers;
pub mod message;
pub mod native_events;
pub mod network;
pub mod panel_actions;
pub mod panel_state;
pub mod panel_update;
pub mod state;
pub mod subscription;
pub mod tray_actions;
pub mod tray_state;
pub mod tray_subscription;
pub mod update;
pub mod view;

#[cfg(test)]
mod bluetooth_tests;
#[cfg(test)]
mod drawers_tests;
#[cfg(test)]
mod palette_tests;
#[cfg(test)]
mod panel_tests;
#[cfg(test)]
mod tray_actions_tests;
#[cfg(test)]
mod tray_activation_tests;
#[cfg(test)]
mod tray_test_support;

use std::sync::Arc;

use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use sparky_theme::Skin;

use crate::app::state::Bar;
use crate::ports::workspaces::SparkyWorkspaces;
use crate::ui::style::{FONT, HEIGHT, NAMESPACE, TEXT_SIZE};

pub fn run() -> Result<(), iced_layershell::Error> {
    iced_layershell::build_pattern::daemon(boot, namespace, update::update, view::view)
        .subscription(subscription::subscription)
        .theme(|bar: &Bar, _window| bar.skin().theme())
        .settings(settings())
        .run()
}

fn boot() -> Bar {
    let skin = Skin::new(sparky_theme::current_palette());
    let now = chrono::Local::now().naive_local();
    Bar::new(skin, Arc::new(SparkyWorkspaces), now)
}

fn namespace() -> String {
    NAMESPACE.to_owned()
}

fn settings() -> Settings {
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Left | Anchor::Right,
            layer: Layer::Top,
            exclusive_zone: HEIGHT as i32,
            size: Some((0, HEIGHT)),
            keyboard_interactivity: KeyboardInteractivity::None,
            start_mode: StartMode::AllScreens,
            ..LayerShellSettings::default()
        },
        default_font: FONT,
        default_text_size: TEXT_SIZE.into(),
        antialiasing: true,
        ..Settings::default()
    }
}

#[cfg(test)]
pub(crate) mod panel_flow_tests;

#[cfg(test)]
mod panel_keyboard_tests;
