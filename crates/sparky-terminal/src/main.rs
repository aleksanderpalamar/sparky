use std::process::ExitCode;

use sparky_terminal::domain::launch::parse;

const USAGE: u8 = 2;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let launch = match parse(&arguments) {
        Ok(launch) => launch,
        Err(error) => {
            eprintln!("sparky-terminal: {error}");
            eprintln!("uso: sparky-terminal [-d PASTA] [-e COMANDO ARGUMENTOS...]");
            return ExitCode::from(USAGE);
        }
    };
    match sparky_terminal::app::run(launch) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sparky-terminal: {error}");
            ExitCode::FAILURE
        }
    }
}
