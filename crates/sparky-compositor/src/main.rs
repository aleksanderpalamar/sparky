use std::process::ExitCode;

fn main() -> ExitCode {
    match sparky_compositor::startup::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sparky: {error}");
            ExitCode::FAILURE
        }
    }
}
