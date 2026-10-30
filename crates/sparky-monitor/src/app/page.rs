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

    pub fn label(self) -> &'static str {
        match self {
            Page::Dashboard => "Painel",
            Page::Applications => "Aplicativos",
            Page::Processes => "Processos",
            Page::Cpu => "CPU",
            Page::Memory => "Memória",
            Page::Disk => "Disco",
            Page::Network => "Rede",
            Page::Gpu => "GPU",
        }
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
