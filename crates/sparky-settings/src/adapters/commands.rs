use std::process::Command;

use crate::ports::commands::{CommandError, CommandRunner};

pub struct RealCommands;

impl CommandRunner for RealCommands {
    fn run(&self, program: &str, arguments: &[&str]) -> Result<String, CommandError> {
        let Ok(output) = Command::new(program).args(arguments).output() else {
            return Err(CommandError::Unavailable);
        };
        if !output.status.success() {
            return Err(CommandError::Rejected {
                code: output.status.code(),
                message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}
