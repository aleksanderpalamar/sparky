use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use sparky_keyring_prompt::domain::mode::Mode;

const CANCELLED: u8 = 1;
const USAGE: u8 = 2;

fn main() -> ExitCode {
    let Some(mode) = Mode::from_args(std::env::args().skip(1)) else {
        eprintln!("uso: sparky-keyring-prompt create | unlock [--retry]");
        return ExitCode::from(USAGE);
    };
    let answered = Arc::new(AtomicBool::new(false));
    if let Err(error) = sparky_keyring_prompt::app::run(mode, Arc::clone(&answered)) {
        eprintln!("sparky-keyring-prompt: {error}");
        return ExitCode::FAILURE;
    }
    match answered.load(Ordering::SeqCst) {
        true => ExitCode::SUCCESS,
        false => ExitCode::from(CANCELLED),
    }
}
