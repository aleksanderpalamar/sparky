use std::process::ExitCode;

use sparky_shot::domain::mode::parse;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments == ["--share-source"] {
        return match sparky_shot::share_source::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("sparky-shot: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let invocation = match parse(&arguments) {
        Ok(invocation) => invocation,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    match sparky_shot::run::run(invocation) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sparky-shot: {error}");
            ExitCode::FAILURE
        }
    }
}
