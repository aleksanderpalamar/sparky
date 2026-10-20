use iced::Color;
use sparky_theme::Skin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
    Info(String),
    Error(String),
}

impl Notice {
    pub fn text(&self) -> &str {
        match self {
            Notice::Info(text) | Notice::Error(text) => text,
        }
    }

    pub fn color(&self, skin: Skin) -> Color {
        match self {
            Notice::Info(_) => skin.foreground,
            Notice::Error(_) => skin.urgent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Notice;
    use sparky_theme::Skin;

    #[test]
    fn erro_aparece_na_cor_de_alerta() {
        let skin = Skin::default();
        assert_eq!(Notice::Error("x".to_owned()).color(skin), skin.urgent);
        assert_eq!(Notice::Info("ok".to_owned()).text(), "ok");
    }
}
