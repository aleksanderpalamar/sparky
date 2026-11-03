pub mod autostart;
pub mod autostart_files;
pub mod components;
pub mod departure;
pub mod environment;
pub mod leaving;
pub mod login_screen;
mod startup;
mod systemd;

#[cfg(test)]
mod autostart_tests;
#[cfg(test)]
mod departure_tests;

use crate::backend::Backend;
use crate::session::environment::DESKTOP;
use crate::state::Sparky;
use sparky_xdg::programs::installed;

impl Sparky {
    pub fn is_real_session(&self) -> bool {
        matches!(self.backend, Backend::Drm(_))
    }

    pub fn start_session(&mut self) {
        let environment = self.child_environment();
        let autostart = autostart::commands(&autostart_files::from_system(), DESKTOP, installed);
        let extra = self.config.autostart.clone();
        let components = components::components(installed);
        std::thread::spawn(move || {
            systemd::prepare(&environment);
            startup::launch(components, autostart.into_iter().chain(extra), &environment)
        });
    }
}
