use std::process::ExitCode;

use sparky_resources::elevation::serve;
use sparky_resources::platform::{LinuxFiles, LinuxPriority};

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let caller = std::env::var("PKEXEC_UID")
        .ok()
        .and_then(|value| value.parse().ok());
    match serve(&arguments, caller, &LinuxFiles, &LinuxPriority) {
        Ok(()) => ExitCode::SUCCESS,
        Err(refusal) => {
            eprintln!("sparky-resources-helper: recusado: {refusal:?}");
            ExitCode::from(refusal.code())
        }
    }
}
