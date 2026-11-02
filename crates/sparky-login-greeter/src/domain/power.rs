#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Power {
    Shutdown,
    Reboot,
    Suspend,
}

impl Power {
    pub const ALL: [Power; 3] = [Power::Shutdown, Power::Reboot, Power::Suspend];

    pub fn label(self) -> &'static str {
        match self {
            Power::Shutdown => "Desligar",
            Power::Reboot => "Reiniciar",
            Power::Suspend => "Suspend",
        }
    }
}
