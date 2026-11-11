use std::process::Command;

pub trait Runner: Send + Sync + std::fmt::Debug {
    fn run(&self, program: &str, args: &[&str]) -> Result<String, String>;
}

#[derive(Debug)]
pub struct Process;

impl Runner for Process {
    fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
        let output = Command::new(program)
            .args(args)
            .env("LC_ALL", "C")
            .output()
            .map_err(|error| error.to_string())?;
        if output.status.success() {
            return String::from_utf8(output.stdout).map_err(|error| error.to_string());
        }
        let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        if output.status.code() == Some(1)
            && program == "pacman"
            && args.first() == Some(&"-Ss")
            && message.is_empty()
        {
            return Ok(String::new());
        }
        if !message.is_empty() {
            return Err(message);
        }
        Err(format!("{program} exited with {}", output.status))
    }
}
