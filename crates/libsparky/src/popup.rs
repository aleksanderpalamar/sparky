use iced::Point;
use iced::window::Id;
use iced_layershell::actions::IcedNewPopupSettings;
use iced_layershell::reexport::{PopupAnchor, PopupGravity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Open,
    Close(Id),
    Keep,
}

pub fn at(parent: Id, point: Point, size: (f32, f32)) -> IcedNewPopupSettings {
    let side = |value: f32| value.round().max(1.0) as u32;
    let position = (point.x.round() as i32, point.y.round() as i32);
    IcedNewPopupSettings {
        anchor: PopupAnchor::TopLeft,
        gravity: PopupGravity::BottomRight,
        ..IcedNewPopupSettings::at_position(parent, (side(size.0), side(size.1)), position)
    }
}

pub fn step(wanted: bool, shown: Option<Id>) -> Step {
    match (wanted, shown) {
        (true, None) => Step::Open,
        (false, Some(id)) => Step::Close(id),
        _ => Step::Keep,
    }
}
