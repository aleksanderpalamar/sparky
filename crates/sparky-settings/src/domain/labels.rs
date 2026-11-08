use crate::domain::field::Field;

impl Field {
    pub fn label(self) -> String {
        crate::i18n::text(&format!("field-{}-label", self.key()))
    }

    pub fn help(self) -> String {
        crate::i18n::text(&format!("field-{}-help", self.key()))
    }

    fn key(self) -> &'static str {
        match self {
            Field::GapsInner => "gaps-inner",
            Field::GapsOuter => "gaps-outer",
            Field::Border => "border",
            Field::FloatWidth => "float-width",
            Field::FloatHeight => "float-height",
            Field::DefaultBinds => "default-binds",
            Field::RepeatRate => "repeat-rate",
            Field::RepeatDelay => "repeat-delay",
            Field::Terminal => "terminal",
            Field::Launcher => "launcher",
            Field::Files => "files",
            Field::Settings => "settings",
            Field::Screenshot => "screenshot",
            Field::Locker => "locker",
            Field::SystemMenu => "system-menu",
            Field::Wallpaper => "wallpaper",
            Field::LoginWallpaper => "login-wallpaper",
        }
    }
}
