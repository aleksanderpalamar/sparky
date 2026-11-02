use std::process::ExitCode;
use std::sync::Arc;

use sparky_login_greeter::adapters::preview::Preview;
use sparky_login_greeter::adapters::session_files;
use sparky_login_greeter::app::{self, Start};
use sparky_login_greeter::cli::{self, Command, Options};
use sparky_login_greeter::domain::sessions::{Sessions, sessions};
use sparky_theme::{locations, wallpapers};
use sparky_xdg::desktop_entry::locale_from_env;
use sparky_xdg::programs::installed;

const USAGE: u8 = 2;

fn main() -> ExitCode {
    let Some(Command::Preview(options)) = cli::parse(std::env::args().skip(1)) else {
        eprintln!("{}", cli::USAGE);
        return ExitCode::from(USAGE);
    };
    match app::run(preview(options)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sparky-login-greeter: {error}");
            ExitCode::FAILURE
        }
    }
}

fn preview(options: Options) -> Start {
    let list = sessions(session_files::from_system(), &locale_from_env(), installed);
    Start {
        backend: Arc::new(Preview),
        sessions: Sessions::new(list, None),
        last_user: std::env::var("USER").ok(),
        wallpaper: options.wallpaper.or_else(default_wallpaper),
        palette: sparky_theme::current_palette(),
    }
}

fn default_wallpaper() -> Option<std::path::PathBuf> {
    wallpapers::of_theme(&wallpapers::roots_from_env(), locations::DEFAULT_THEME)
        .into_iter()
        .next()
}
