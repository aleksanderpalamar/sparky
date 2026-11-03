use std::thread;

use smithay::reexports::calloop::channel::{Event, channel};

use crate::bindings::Bindings;
use crate::spawn::start;
use crate::state::Sparky;

const SHELL: &str = "sh";

pub fn login_screen_bindings() -> Bindings {
    Bindings::default()
}

impl Sparky {
    pub fn enter_login_screen(&mut self) {
        self.bindings = login_screen_bindings();
    }

    pub fn start_login_screen(&mut self, greeter: &str) {
        let (sender, receiver) = channel::<()>();
        let watched = self.loop_handle.insert_source(receiver, |event, _, state| {
            if let Event::Msg(()) = event {
                tracing::info!("a tela de login fechou");
                state.loop_signal.stop();
            }
        });
        if let Err(error) = watched {
            tracing::error!("não deu para acompanhar a tela de login: {error}");
            self.loop_signal.stop();
            return;
        }
        let arguments = ["-c".to_owned(), greeter.to_owned()];
        match start(SHELL, &arguments, &self.child_environment()) {
            Ok(mut child) => {
                thread::spawn(move || {
                    let _ = child.wait();
                    let _ = sender.send(());
                });
            }
            Err(error) => {
                tracing::error!("a tela de login não abriu: {error}");
                self.loop_signal.stop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use smithay::input::keyboard::Keysym;
    use sparky_config::Config;
    use sparky_config::combo::Mods;

    use super::login_screen_bindings;
    use crate::config::bindings_for;

    const SUPER: Mods = Mods {
        logo: true,
        shift: false,
        ctrl: false,
        alt: false,
    };

    #[test]
    fn na_tela_de_login_o_atalho_do_terminal_nao_existe() {
        let desktop = bindings_for(&Config::default());
        assert!(desktop.find(SUPER, 0, Some(Keysym::Return)).is_some());
        assert!(
            login_screen_bindings()
                .find(SUPER, 0, Some(Keysym::Return))
                .is_none()
        );
    }
}
