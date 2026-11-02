pub use sparky_login_core::protocol::Power;

pub fn label(power: Power) -> &'static str {
    match power {
        Power::Shutdown => "Desligar",
        Power::Reboot => "Reiniciar",
        Power::Suspend => "Suspend",
    }
}
