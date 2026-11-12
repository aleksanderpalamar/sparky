use std::process::Command;

pub trait Runner: Send + Sync + std::fmt::Debug {
    fn run(&self, program: &str, args: &[&str]) -> Result<String, String>;

    fn run_status(&self, program: &str, args: &[&str]) -> Result<CommandOutput, String> {
        self.run(program, args).map(|stdout| CommandOutput {
            stdout,
            stderr: String::new(),
            code: Some(0),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub code: Option<i32>,
}

#[derive(Debug)]
pub struct Process;

impl Runner for Process {
    fn run(&self, program: &str, args: &[&str]) -> Result<String, String> {
        let output = self.run_status(program, args)?;
        if output.code == Some(0) {
            return Ok(output.stdout);
        }
        let message = output.stderr.trim();
        if output.code == Some(1)
            && program == "pacman"
            && args.first() == Some(&"-Ss")
            && message.is_empty()
        {
            return Ok(String::new());
        }
        if !message.is_empty() {
            return Err(message.to_owned());
        }
        Err(format!("{program} exited with {:?}", output.code))
    }

    fn run_status(&self, program: &str, args: &[&str]) -> Result<CommandOutput, String> {
        let output = Command::new(program)
            .args(args)
            .env("LC_ALL", "C")
            .output()
            .map_err(|error| error.to_string())?;
        Ok(CommandOutput {
            stdout: String::from_utf8(output.stdout).map_err(|error| error.to_string())?,
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            code: output.status.code(),
        })
    }
}
