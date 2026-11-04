use std::process::ExitCode;

use sparky_launcher::domain::mode::Mode;
use sparky_launcher::i18n;

const USAGE: u8 = 2;

fn main() -> ExitCode {
    let Some(mode) = Mode::from_args(std::env::args().skip(1)) else {
        eprintln!("{}", i18n::text("usage"));
        return ExitCode::from(USAGE);
    };
    match sparky_launcher::app::run(mode) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sparky-launcher: {error}");
            ExitCode::FAILURE
        }
    }
}
