use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::thread;

use crate::domain::app::Launch;
use crate::ports::Starter;

const FALLBACK_TERMINAL: &str = "sparky-terminal";

pub struct Spawner;

impl Starter for Spawner {
    fn start(&self, launch: &Launch) -> Result<(), String> {
        let terminal = std::env::var("TERMINAL")
            .ok()
            .filter(|name| !name.is_empty());
        let words = command(launch, terminal.as_deref().unwrap_or(FALLBACK_TERMINAL));
        let Some((program, arguments)) = words.split_first() else {
            return Err("comando vazio".to_owned());
        };
        let mut child = Command::new(program)
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()
            .map_err(|error| format!("{program}: {error}"))?;
        thread::spawn(move || child.wait());
        Ok(())
    }
}

pub fn command(launch: &Launch, terminal: &str) -> Vec<String> {
    match launch.terminal {
        true => [terminal.to_owned(), "-e".to_owned()]
            .into_iter()
            .chain(launch.arguments.iter().cloned())
            .collect(),
        false => launch.arguments.clone(),
    }
}
