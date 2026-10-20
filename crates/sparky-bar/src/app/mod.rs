pub mod message;
pub mod state;
pub mod subscription;
pub mod update;
pub mod view;

use std::sync::Arc;

use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use sparky_theme::Skin;

use crate::app::state::Bar;
use crate::ports::workspaces::SparkyWorkspaces;
use crate::ui::style::{FONT, HEIGHT, NAMESPACE, TEXT_SIZE};
use crate::ui::theme;

pub fn run() -> Result<(), iced_layershell::Error> {
    iced_layershell::build_pattern::daemon(boot, namespace, update::update, view::view)
        .subscription(subscription::subscription)
        .theme(|bar: &Bar, _window| theme::theme(bar.skin()))
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
