use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use nix::unistd::{User, getuid};
use sparky_lock::adapters::appearance;
use sparky_lock::adapters::pam::PamAuthenticator;
use sparky_lock::app::{self, Start};
use sparky_pam::LinuxPam;
use sparky_theme::{locations, wallpapers};

fn main() -> ExitCode {
    let Some(user) = User::from_uid(getuid()).ok().flatten() else {
        eprintln!("sparky-lock: não deu para saber quem é o usuário da sessão");
        return ExitCode::FAILURE;
    };
    let config_home = std::env::var("XDG_CONFIG_HOME").ok();
    let config = sparky_config::path(config_home.as_deref(), &user.dir.to_string_lossy());
    let start = Start {
        wallpaper: appearance::wallpaper(&config, &user.dir, default_wallpaper),
        palette: sparky_theme::current_palette(),
        authenticator: Arc::new(PamAuthenticator::new(
            LinuxPam::default(),
            user.name.clone(),
        )),
        user: user.name,
    };
    match app::run(start) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sparky-lock: {error}");
            ExitCode::FAILURE
        }
    }
}

fn default_wallpaper() -> Option<PathBuf> {
    wallpapers::of_theme(&wallpapers::roots_from_env(), locations::DEFAULT_THEME)
        .into_iter()
        .next()
}
