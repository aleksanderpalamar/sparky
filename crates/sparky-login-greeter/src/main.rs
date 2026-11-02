use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use sparky_login_core::protocol::SOCKET_VARIABLE;
use sparky_login_core::session_files;
use sparky_login_core::sessions::{Session, Sessions, sessions};
use sparky_login_greeter::adapters::daemon::Daemon;
use sparky_login_greeter::adapters::preview::Preview;
use sparky_login_greeter::app::{self, Mode, Start};
use sparky_login_greeter::cli::{self, Command, Options};
use sparky_theme::{locations, wallpapers};
use sparky_xdg::desktop_entry::locale_from_env;
use sparky_xdg::programs::installed;

const USAGE: u8 = 2;
const NO_SOCKET: &str =
    "sem SPARKY_LOGIN_SOCKET; a tela abre pelo serviço sparky-login, ou use --preview";

fn main() -> ExitCode {
    let start = match cli::parse(std::env::args().skip(1)) {
        Some(Command::Preview(options)) => Ok(preview(options)),
        Some(Command::Greeter) => greeter(),
        None => {
            eprintln!("{}", cli::USAGE);
            return ExitCode::from(USAGE);
        }
    };
    let result = start.and_then(|start| app::run(start).map_err(|error| error.to_string()));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sparky-login-greeter: {error}");
            ExitCode::FAILURE
        }
    }
}

fn preview(options: Options) -> Start {
    Start {
        mode: Mode::Preview,
        backend: Arc::new(Preview),
        sessions: Sessions::new(installed_sessions(), None),
        last_user: std::env::var("USER").ok(),
        wallpaper: options.wallpaper.or_else(default_wallpaper),
        palette: sparky_theme::current_palette(),
    }
}

fn greeter() -> Result<Start, String> {
    let socket = std::env::var_os(SOCKET_VARIABLE).ok_or(NO_SOCKET)?;
    let daemon = Daemon::connect(Path::new(&socket))?;
    let (last_user, last_session) = daemon.hello()?;
    Ok(Start {
        mode: Mode::Greeter,
        backend: Arc::new(daemon),
        sessions: Sessions::new(installed_sessions(), last_session.as_deref()),
        last_user,
        wallpaper: default_wallpaper(),
        palette: sparky_theme::current_palette(),
    })
}

fn installed_sessions() -> Vec<Session> {
    sessions(session_files::from_system(), &locale_from_env(), installed)
}

fn default_wallpaper() -> Option<PathBuf> {
    wallpapers::of_theme(&wallpapers::roots_from_env(), locations::DEFAULT_THEME)
        .into_iter()
        .next()
}
