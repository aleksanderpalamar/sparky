#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelKind {
    Clock,
    Audio,
    Network,
    Bluetooth,
}
