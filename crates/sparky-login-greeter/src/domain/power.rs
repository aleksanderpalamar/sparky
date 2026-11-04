pub use sparky_login_core::protocol::Power;

pub fn key(power: Power) -> &'static str {
    match power {
        Power::Shutdown => "power-shutdown",
        Power::Reboot => "power-reboot",
        Power::Suspend => "power-suspend",
    }
}
