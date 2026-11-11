#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelKind {
    Clock,
    Audio,
    Network,
    Bluetooth,
    Agents,
}

impl PanelKind {
    pub fn size(self) -> (u32, u32) {
        match self {
            Self::Clock => (760, 520),
            Self::Audio => (420, 500),
            Self::Network => (420, 450),
            Self::Bluetooth => (420, 380),
            Self::Agents => (420, 460),
        }
    }
}
