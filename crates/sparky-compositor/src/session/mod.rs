pub mod autostart;
pub mod autostart_files;
pub mod components;
pub mod desktop_entry;
pub mod environment;

#[cfg(test)]
mod autostart_tests;
#[cfg(test)]
mod desktop_entry_tests;

use crate::apps::installed;
use crate::backend::Backend;
use crate::session::environment::{DESKTOP, activation_arguments};
use crate::spawn::{spawn, spawn_program};
use crate::state::Sparky;

const ACTIVATION: &str = "dbus-update-activation-environment";

impl Sparky {
    pub fn is_real_session(&self) -> bool {
        matches!(self.backend, Backend::Drm(_))
    }

    pub fn start_session(&mut self) {
        let environment = self.child_environment();
        spawn_program(ACTIVATION, &activation_arguments(&environment.variables()), &environment);
        let autostart = autostart::commands(&autostart_files::from_system(), DESKTOP, installed);
        for command in components::components(installed).into_iter().chain(autostart) {
            tracing::info!("sessão abre {command}");
            spawn(&command, &environment);
        }
    }
}
