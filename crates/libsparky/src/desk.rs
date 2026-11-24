use iced::window::Id;
use iced_layershell::actions::IcedXdgWindowSettings;
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};

pub fn settings() -> Settings {
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::empty(),
            start_mode: StartMode::Background,
            ..LayerShellSettings::default()
        },
        default_font: sparky_ui::style::FONT,
        antialiasing: true,
        ..Settings::default()
    }
}

pub fn window(
    app_id: &'static str,
    size: (u32, u32),
    min_size: (u32, u32),
) -> IcedXdgWindowSettings {
    IcedXdgWindowSettings {
        size: Some(size),
        client_side_decorations: false,
        app_id: Some(app_id),
        min_size: Some(min_size),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Desk {
    main: Id,
}

impl Desk {
    pub fn new(main: Id) -> Self {
        Self { main }
    }

    pub fn main(&self) -> Id {
        self.main
    }

    pub fn is_main(&self, id: Id) -> bool {
        self.main == id
    }

    pub fn ends_with(&self, closed: Id) -> bool {
        self.is_main(closed)
    }
}
