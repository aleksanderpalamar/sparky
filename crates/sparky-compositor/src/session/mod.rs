pub mod autostart;
pub mod autostart_files;
pub mod components;
pub mod departure;
pub mod environment;
pub mod leaving;
mod startup;

#[cfg(test)]
mod autostart_tests;
#[cfg(test)]
mod departure_tests;

use crate::backend::Backend;
use crate::session::environment::{DESKTOP, activation_arguments};
use crate::spawn::spawn_program;
use crate::state::Sparky;
use sparky_xdg::programs::installed;

const ACTIVATION: &str = "dbus-update-activation-environment";

impl Sparky {
    pub fn is_real_session(&self) -> bool {
        matches!(self.backend, Backend::Drm(_))
    }

    pub fn start_session(&mut self) {
        let environment = self.child_environment();
        spawn_program(
            ACTIVATION,
            &activation_arguments(&environment.variables()),
            &environment,
        );
        let autostart = autostart::commands(&autostart_files::from_system(), DESKTOP, installed);
        let extra = self.config.autostart.clone();
        let components = components::components(installed);
        std::thread::spawn(move || {
            startup::launch(components, autostart.into_iter().chain(extra), &environment)
        });
    }
}
