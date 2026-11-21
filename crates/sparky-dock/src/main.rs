use std::process::ExitCode;

fn main() -> ExitCode {
    match sparky_dock::app::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sparky-dock: {error}");
            ExitCode::FAILURE
        }
    }
}
