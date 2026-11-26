use std::process::ExitCode;

fn main() -> ExitCode {
    match sparky_obs_guide::app::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("sparky-obs-guide: {error}");
            ExitCode::FAILURE
        }
    }
}
