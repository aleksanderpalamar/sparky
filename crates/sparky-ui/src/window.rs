use iced::Size;
use iced::window::{Settings, settings::PlatformSpecific};

pub fn settings(application_id: &str, size: Size) -> Settings {
    Settings {
        size,
        platform_specific: PlatformSpecific {
            application_id: application_id.to_owned(),
            ..PlatformSpecific::default()
        },
        transparent: true,
        ..Settings::default()
    }
}

#[cfg(test)]
mod tests {
    use iced::Size;

    #[test]
    fn as_janelas_aceitam_fundo_translucido_para_o_frosted_glass() {
        let settings = super::settings("sparky-teste", Size::new(800.0, 600.0));
        assert!(settings.transparent);
        assert_eq!(settings.platform_specific.application_id, "sparky-teste");
    }
}
