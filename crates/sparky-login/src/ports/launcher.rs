use std::path::Path;

use crate::domain::environment::Account;

pub trait Process {
    fn wait(&mut self) -> Result<(), String>;
}

pub trait Launcher {
    fn launch(
        &self,
        account: &Account,
        command: &[String],
        environment: &[(String, String)],
        tty: &Path,
    ) -> Result<Box<dyn Process>, String>;
}
