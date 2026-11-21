use iced_layershell::reexport::{
    Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings, OutputOption,
};

use crate::domain::layout::Geometry;

pub const DOCK: &str = "sparky-dock";
pub const PICKER: &str = "sparky-dock-picker";

pub fn dock(geometry: Geometry) -> NewLayerShellSettings {
    NewLayerShellSettings {
        size: Some(geometry.size),
        layer: Layer::Top,
        anchor: Anchor::Bottom,
        exclusive_zone: Some(geometry.zone),
        margin: Some(margin(geometry)),
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

pub fn margin(geometry: Geometry) -> (i32, i32, i32, i32) {
    (0, 0, geometry.lift, 0)
}
