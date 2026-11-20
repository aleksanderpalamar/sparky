use std::process::{Command, Stdio};
use std::thread;

use crate::ports::Notifier;

pub struct DesktopNotifier {
    pub application: String,
}

impl Notifier for DesktopNotifier {
    fn notify(&self, title: &str, body: &str) {
        let mut command = Command::new("notify-send");
        command
            .args(["-a", &self.application, title, body])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        thread::spawn(move || {
            if let Err(error) = command.status() {
                eprintln!("sparky-resources: sem notificação: {error}");
            }
        });
    }
}
