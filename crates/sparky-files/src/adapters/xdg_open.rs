use std::io;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

use crate::ports::Opener;

pub struct XdgOpen;

impl Opener for XdgOpen {
    fn open(&self, path: &Path) -> io::Result<()> {
        let mut child = Command::new("xdg-open")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        thread::spawn(move || child.wait());
        Ok(())
    }
}
