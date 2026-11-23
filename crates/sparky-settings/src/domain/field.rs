use crate::domain::section::Section;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    GapsInner,
    GapsOuter,
    Border,
    FloatWidth,
    FloatHeight,
    DefaultBinds,
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
    LoginWallpaper,
    DockEnabled,
    DockSize,
    DockAutohide,
    DockRounded,
    DockRadius,
    FrostedGlass,
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

const APPEARANCE: [Field; 8] = [
    Field::GapsInner,
    Field::GapsOuter,
    Field::Border,
    Field::FloatWidth,
    Field::FloatHeight,
    Field::DefaultBinds,
    Field::Wallpaper,
    Field::FrostedGlass,
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

const DOCK: [Field; 5] = [
    Field::DockEnabled,
    Field::DockSize,
    Field::DockAutohide,
    Field::DockRounded,
    Field::DockRadius,
];

const fn bounds(min: i32, max: i32, step: i32) -> Kind {
    Kind::Number(Bounds { min, max, step })
}

impl Field {
    pub fn of(section: Section) -> &'static [Field] {
        match section {
            Section::Appearance => &APPEARANCE,
            Section::Apps => &APPS,
            Section::Dock => &DOCK,
            Section::Autostart
            | Section::Shortcuts
            | Section::Monitors
            | Section::Network
            | Section::Performance
            | Section::Power
            | Section::DateTime
            | Section::Region
            | Section::Users
            | Section::Media
            | Section::Themes
            | Section::About => &[],
        }
    }

    pub fn shown(section: Section, config: &sparky_config::Config) -> Vec<Field> {
        Field::of(section)
            .iter()
            .copied()
            .filter(|field| field.visible(config))
            .collect()
    }

    fn visible(self, config: &sparky_config::Config) -> bool {
        match self {
            Field::DockRadius => config.dock.rounded,
            _ => true,
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
            Field::DockSize => bounds(
                sparky_config::dock::MIN_SIZE,
                sparky_config::dock::MAX_SIZE,
                4,
            ),
            Field::DockRadius => bounds(0, sparky_config::dock::MAX_RADIUS, 1),
            Field::DefaultBinds
            | Field::DockEnabled
            | Field::DockAutohide
            | Field::DockRounded
            | Field::FrostedGlass => Kind::Flag,
            Field::Terminal | Field::Wallpaper | Field::LoginWallpaper => {
                Kind::Text(Blank::Allowed)
            }
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
            Field::LoginWallpaper => ("login", "wallpaper"),
            Field::DockEnabled => ("dock", "enabled"),
            Field::DockSize => ("dock", "size"),
            Field::DockAutohide => ("dock", "autohide"),
            Field::DockRounded => ("dock", "rounded"),
            Field::DockRadius => ("dock", "radius"),
            Field::FrostedGlass => ("general", "frosted_glass"),
        }
    }
}
