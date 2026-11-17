pub mod editing;
pub mod keys;
pub mod message;
pub mod navigation;
pub mod pictures;
pub mod state;
pub mod update;
pub mod view;

#[cfg(test)]
mod editing_tests;
#[cfg(test)]
mod keys_tests;
#[cfg(test)]
mod menu_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod test_support;

use iced::event::{self, Event, Status};
use iced::{Subscription, Task, window};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use sparky_theme::Skin;
use sparky_ui::press::{Press, press};
use sparky_ui::surface::transparent;

use crate::adapters::catalog::installed_apps;
use crate::adapters::icons::Icons;
use crate::adapters::instance::{Claim, Instance, claim, socket_from_env};
use crate::adapters::library_files::LibraryFiles;
use crate::app::keys::{Command, command, confirms, typed};
use crate::app::message::Message;
use crate::app::state::Launcher;
use crate::app::update::update;
use crate::domain::app::App;
use crate::domain::mode::Mode;
use crate::domain::symbols::symbols;
use crate::domain::system::entries;

const NAMESPACE: &str = "sparky-launcher";

pub fn run(mode: Mode) -> Result<(), iced_layershell::Error> {
    match socket_from_env(mode).map(|path| claim(&path)) {
        Some(Ok(Claim::Signalled)) => return Ok(()),
        Some(Ok(Claim::Owner(instance))) => close_on_request(instance),
        Some(Err(error)) => {
            eprintln!("sparky-launcher: não deu para controlar a abertura única: {error}")
        }
        None => {}
    }
    let apps = match mode {
        Mode::Apps => installed_apps(),
        Mode::System => entries(),
    };
    let boot = move || {
        let launcher = Launcher::new(
            apps.clone(),
            Skin::new(sparky_theme::current_palette()),
            mode,
            Box::new(LibraryFiles::from_env()),
        );
        (launcher, icons(apps.clone()))
    };
    iced_layershell::application(boot, || NAMESPACE.to_owned(), update, view::view)
        .subscription(|_| {
            Subscription::batch([
                event::listen_with(keyboard_message),
                sparky_ui::palette::subscription().map(Message::Palette),
            ])
        })
        .theme(|launcher: &Launcher| launcher.skin.theme())
        .style(transparent)
        .settings(settings())
        .run()
}

fn close_on_request(instance: Instance) {
    std::thread::spawn(move || {
        if instance.wait().is_ok() {
            std::process::exit(0);
        }
    });
}

fn icons(apps: Vec<App>) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let icons = Icons::current();
                let mut found = icons.resolve(&apps);
                found.extend(icons.named(&symbols()));
                found
            })
            .await
            .unwrap_or_default()
        },
        Message::Icons,
    )
}

fn keyboard_message(event: Event, status: Status, _window: window::Id) -> Option<Message> {
    if let Some(text) = typed(&event, status) {
        return Some(Message::TypeAhead(text));
    }
    match press(event, status)? {
        Press::Escape => Some(Message::Close),
        Press::Key(key, _) if confirms(&key) => Some(Message::Submit),
        Press::Key(key, modifiers) => match command(&key, modifiers)? {
            Command::Move(step) => Some(Message::Move(step)),
            Command::Shelf(delta) => Some(Message::NextShelf(delta)),
            Command::Menu => Some(Message::MenuKey),
        },
    }
}

fn settings() -> Settings {
    Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            layer: Layer::Overlay,
            exclusive_zone: -1,
            size: None,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            ..LayerShellSettings::default()
        },
        antialiasing: true,
        default_font: sparky_ui::style::FONT,
        ..Settings::default()
    }
}

#[cfg(test)]
mod tests {
    use iced_layershell::reexport::Anchor;

    use super::settings;

    #[test]
    fn a_camada_cobre_a_tela_toda_para_o_clique_fora_fechar() {
        let camada = settings().layer_settings;
        assert_eq!(
            camada.anchor,
            Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right
        );
        assert_eq!(camada.exclusive_zone, -1);
        assert_eq!(camada.size, None);
    }
}
