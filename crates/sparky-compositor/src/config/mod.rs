pub mod disk;
pub mod keyboard;
mod watch;

use std::time::SystemTime;

use sparky_config::{Config, ConfigError, binds};

use crate::apps::detected_terminal;
use crate::bindings::Bindings;
use crate::input::keyboard_layout::KeyboardLayout;
use crate::state::Sparky;

pub fn initial() -> (Config, Option<SystemTime>) {
    let snapshot = disk::read(&disk::path());
    match disk::interpret(snapshot.contents.as_deref()) {
        Ok(config) => (config, snapshot.stamp),
        Err(error) => {
            tracing::warn!("{error}; usando a configuração padrão do sparky");
            (Config::default(), snapshot.stamp)
        }
    }
}

pub fn bindings_for(config: &Config) -> Bindings {
    let terminal = detected_terminal(config.apps.terminal.as_deref());
    Bindings::new(binds::merged(config, &terminal))
}

pub fn keyboard_for(config: &Config) -> KeyboardLayout {
    let system = KeyboardLayout::from_system(|path| std::fs::read_to_string(path).ok());
    keyboard::layout(&config.input, system)
}

impl Sparky {
    pub fn reload_config(&mut self) -> Result<(), ConfigError> {
        let snapshot = disk::read(&disk::path());
        self.config_stamp = snapshot.stamp;
        let config = disk::interpret(snapshot.contents.as_deref())?;
        self.apply_config(config);
        tracing::info!("configuração recarregada");
        Ok(())
    }

    fn apply_config(&mut self, config: Config) {
        self.bindings = bindings_for(&config);
        self.desktop.set_style(config.general.style());
        if config.input != self.config.input {
            self.apply_keyboard(&config);
        }
        self.config = config;
        self.relayout();
    }

    fn apply_keyboard(&mut self, config: &Config) {
        let Some(handle) = self.seat.get_keyboard() else {
            return;
        };
        let layout = keyboard_for(config);
        if let Err(error) = handle.set_xkb_config(self, keyboard::xkb_config(&layout)) {
            tracing::warn!("layout de teclado recusado: {error:?}");
        }
        handle.change_repeat_info(config.input.repeat_rate, config.input.repeat_delay);
    }

    pub fn border(&self) -> i32 {
        self.config.general.border
    }
}
