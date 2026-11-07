use sparky_config::Greeting;

use crate::domain::launch::{Launch, Program};

pub const PROGRAM: &str = "sparky-fetch";

pub fn wanted(launch: &Launch, preference: impl FnOnce() -> Greeting) -> Greeting {
    match launch.program {
        Program::Shell => preference(),
        Program::Command { .. } => Greeting::Quiet,
    }
}

pub fn arguments(columns: usize) -> Vec<String> {
    let width = columns.max(1).to_string();
    ["--format", "visual", "--color", "always", "--width", &width]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

pub fn screen_text(output: &[u8]) -> Vec<u8> {
    let mut text = Vec::with_capacity(output.len() + output.len() / 16);
    let mut previous = None;
    for byte in output {
        if *byte == b'\n' && previous != Some(b'\r') {
            text.push(b'\r');
        }
        text.push(*byte);
        previous = Some(*byte);
    }
    text
}
