use crate::domain::table::Column;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Applications,
    Processes,
    Cpu,
    Memory,
    Disk,
    Network,
    Gpu,
}

impl Page {
    pub const ALL: [Page; 8] = [
        Page::Dashboard,
        Page::Applications,
        Page::Processes,
        Page::Cpu,
        Page::Memory,
        Page::Disk,
        Page::Network,
        Page::Gpu,
    ];

    pub fn nth(index: usize) -> Option<Page> {
        Page::ALL.get(index).copied()
    }

    pub fn label(self) -> String {
        crate::i18n::text(match self {
            Page::Dashboard => "dashboard",
            Page::Applications => "applications",
            Page::Processes => "processes",
            Page::Cpu => "cpu",
            Page::Memory => "memory",
            Page::Disk => "disk",
            Page::Network => "network",
            Page::Gpu => "gpu",
        })
    }

    pub fn resource(self) -> Option<Column> {
        match self {
            Page::Cpu => Some(Column::Cpu),
            Page::Memory => Some(Column::Memory),
            Page::Disk => Some(Column::Disk),
            Page::Gpu => Some(Column::Gpu),
            Page::Dashboard | Page::Applications | Page::Processes | Page::Network => None,
        }
    }
}
