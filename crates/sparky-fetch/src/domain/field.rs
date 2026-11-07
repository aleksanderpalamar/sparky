use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    Os,
    Kernel,
    Uptime,
    Cpu,
    Gpu,
    Memory,
    Disk,
    Terminal,
    Shell,
    Packages,
    Compositor,
    Session,
    Monitors,
    Resolution,
    Theme,
    Icons,
    Font,
    Colors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Linux,
    Sparky,
}

impl Field {
    pub const ALL: [Field; 18] = [
        Field::Os,
        Field::Kernel,
        Field::Uptime,
        Field::Cpu,
        Field::Gpu,
        Field::Memory,
        Field::Disk,
        Field::Terminal,
        Field::Shell,
        Field::Packages,
        Field::Compositor,
        Field::Session,
        Field::Monitors,
        Field::Resolution,
        Field::Theme,
        Field::Icons,
        Field::Font,
        Field::Colors,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Field::Os => "os",
            Field::Kernel => "kernel",
            Field::Uptime => "uptime",
            Field::Cpu => "cpu",
            Field::Gpu => "gpu",
            Field::Memory => "memory",
            Field::Disk => "disk",
            Field::Terminal => "terminal",
            Field::Shell => "shell",
            Field::Packages => "packages",
            Field::Compositor => "compositor",
            Field::Session => "session",
            Field::Monitors => "monitors",
            Field::Resolution => "resolution",
            Field::Theme => "theme",
            Field::Icons => "icons",
            Field::Font => "font",
            Field::Colors => "colors",
        }
    }

    pub fn named(key: &str) -> Option<Field> {
        let key = key.trim();
        Self::ALL.into_iter().find(|field| field.key() == key)
    }

    pub fn group(self) -> Group {
        match self {
            Field::Os
            | Field::Kernel
            | Field::Uptime
            | Field::Cpu
            | Field::Gpu
            | Field::Memory
            | Field::Disk
            | Field::Terminal
            | Field::Shell
            | Field::Packages => Group::Linux,
            Field::Compositor
            | Field::Session
            | Field::Monitors
            | Field::Resolution
            | Field::Theme
            | Field::Icons
            | Field::Font
            | Field::Colors => Group::Sparky,
        }
    }
}
