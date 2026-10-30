use crate::domain::section::Section;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    GapsInner,
    GapsOuter,
    Border,
    FloatWidth,
    FloatHeight,
    DefaultBinds,
    Layout,
    Variant,
    Options,
    RepeatRate,
    RepeatDelay,
    Terminal,
    Launcher,
    Files,
    Settings,
    Screenshot,
    Locker,
    SystemMenu,
    Wallpaper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    pub min: i32,
    pub max: i32,
    pub step: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blank {
    Allowed,
    Refused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Number(Bounds),
    Flag,
    Text(Blank),
}

const APPEARANCE: [Field; 7] = [
    Field::GapsInner,
    Field::GapsOuter,
    Field::Border,
    Field::FloatWidth,
    Field::FloatHeight,
    Field::DefaultBinds,
    Field::Wallpaper,
];
const KEYBOARD: [Field; 5] = [
    Field::Layout,
    Field::Variant,
    Field::Options,
    Field::RepeatRate,
    Field::RepeatDelay,
];
const APPS: [Field; 7] = [
    Field::Terminal,
    Field::Launcher,
    Field::Files,
    Field::Settings,
    Field::Screenshot,
    Field::Locker,
    Field::SystemMenu,
];

const fn bounds(min: i32, max: i32, step: i32) -> Kind {
    Kind::Number(Bounds { min, max, step })
}

impl Field {
    pub fn of(section: Section) -> &'static [Field] {
        match section {
            Section::Appearance => &APPEARANCE,
            Section::Keyboard => &KEYBOARD,
            Section::Apps => &APPS,
            Section::Autostart
            | Section::Shortcuts
            | Section::Monitors
            | Section::Network
            | Section::Power
            | Section::DateTime
            | Section::Region
            | Section::Users
            | Section::Media
            | Section::Themes
            | Section::About => &[],
        }
    }

    pub fn kind(self) -> Kind {
        match self {
            Field::GapsInner => bounds(0, 50, 1),
            Field::GapsOuter => bounds(0, 100, 1),
            Field::Border => bounds(0, 10, 1),
            Field::FloatWidth => bounds(200, 4000, 25),
            Field::FloatHeight => bounds(150, 3000, 25),
            Field::RepeatRate => bounds(1, 100, 5),
            Field::RepeatDelay => bounds(100, 1000, 25),
            Field::DefaultBinds => Kind::Flag,
            Field::Layout
            | Field::Variant
            | Field::Options
            | Field::Terminal
            | Field::Wallpaper => Kind::Text(Blank::Allowed),
            Field::Launcher
            | Field::Files
            | Field::Settings
            | Field::Screenshot
            | Field::Locker
            | Field::SystemMenu => Kind::Text(Blank::Refused),
        }
    }

    pub fn place(self) -> (&'static str, &'static str) {
        match self {
            Field::GapsInner => ("general", "gaps_inner"),
            Field::GapsOuter => ("general", "gaps_outer"),
            Field::Border => ("general", "border"),
            Field::FloatWidth => ("general", "float_width"),
            Field::FloatHeight => ("general", "float_height"),
            Field::DefaultBinds => ("general", "default_binds"),
            Field::Layout => ("input", "layout"),
            Field::Variant => ("input", "variant"),
            Field::Options => ("input", "options"),
            Field::RepeatRate => ("input", "repeat_rate"),
            Field::RepeatDelay => ("input", "repeat_delay"),
            Field::Terminal => ("apps", "terminal"),
            Field::Launcher => ("apps", "launcher"),
            Field::Files => ("apps", "files"),
            Field::Settings => ("apps", "settings"),
            Field::Screenshot => ("apps", "screenshot"),
            Field::Locker => ("apps", "locker"),
            Field::SystemMenu => ("apps", "system_menu"),
            Field::Wallpaper => ("general", "wallpaper"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Blank, Field, Kind};
    use crate::domain::section::Section;

    #[test]
    fn so_as_secoes_de_configuracao_tem_campos() {
        assert_eq!(Field::of(Section::Appearance).len(), 7);
        assert_eq!(Field::Wallpaper.place(), ("general", "wallpaper"));
        assert_eq!(Field::Wallpaper.kind(), Kind::Text(Blank::Allowed));
        assert_eq!(Field::of(Section::Keyboard)[0], Field::Layout);
        assert!(Field::of(Section::About).is_empty());
    }

    #[test]
    fn apps_obrigatorios_nao_aceitam_vazio_e_o_terminal_aceita() {
        assert_eq!(Field::Launcher.kind(), Kind::Text(Blank::Refused));
        assert_eq!(Field::Terminal.kind(), Kind::Text(Blank::Allowed));
    }

    #[test]
    fn o_menu_do_sistema_fica_nos_aplicativos() {
        assert!(Field::of(Section::Apps).contains(&Field::SystemMenu));
        assert_eq!(Field::SystemMenu.place(), ("apps", "system_menu"));
        assert_eq!(Field::SystemMenu.kind(), Kind::Text(Blank::Refused));
    }

    #[test]
    fn cada_campo_mora_numa_chave_do_sparky_toml() {
        assert_eq!(Field::RepeatDelay.place(), ("input", "repeat_delay"));
        assert_eq!(Field::DefaultBinds.place(), ("general", "default_binds"));
    }
}
