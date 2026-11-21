use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::domain::layout::{LIFT, reserved, size};

pub const DOCK: &str = "sparky-dock";
pub const PICKER: &str = "sparky-dock-picker";

pub fn dock(pinned: usize) -> NewLayerShellSettings {
    NewLayerShellSettings {
        size: Some(size(pinned)),
        layer: Layer::Top,
        anchor: Anchor::Bottom,
        exclusive_zone: Some(reserved()),
        margin: Some((0, 0, LIFT, 0)),
        keyboard_interactivity: KeyboardInteractivity::None,
        output_option: OutputOption::Active,
        events_transparent: false,
        namespace: Some(DOCK.to_owned()),
    }
}

pub fn picker() -> NewLayerShellSettings {
    NewLayerShellSettings {
        size: None,
        layer: Layer::Overlay,
        anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
        exclusive_zone: None,
        margin: None,
        keyboard_interactivity: KeyboardInteractivity::Exclusive,
        output_option: OutputOption::Active,
        events_transparent: false,
        namespace: Some(PICKER.to_owned()),
    }
}
