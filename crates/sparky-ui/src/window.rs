use iced::window::{Settings, settings::PlatformSpecific};
use iced::Size;

pub fn settings(application_id: &str, size: Size) -> Settings {
    Settings {
        size,
        platform_specific: PlatformSpecific {
            application_id: application_id.to_owned(),
            ..PlatformSpecific::default()
        },
        ..Settings::default()
    }
}
