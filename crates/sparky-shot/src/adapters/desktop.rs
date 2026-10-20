use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use sparky_xdg::{UserDir, UserDirs};

use crate::domain::pictures::FOLDER;
use crate::ports::{Clipboard, Notifier};

const APP_NAME: &str = "Capturador de tela";

pub struct WlCopy;

impl Clipboard for WlCopy {
    fn copy_png(&self, path: &Path) -> Result<(), String> {
        let image = File::open(path).map_err(|error| error.to_string())?;
        let status = Command::new("wl-copy")
            .args(["--type", "image/png"])
            .stdin(image)
            .stdout(Stdio::null())
            .status()
            .map_err(|error| error.to_string())?;
        match status.success() {
            true => Ok(()),
            false => Err(format!("wl-copy terminou com {status}")),
        }
    }
}

pub struct NotifySend;

impl Notifier for NotifySend {
    fn saved(&self, path: &Path, copied: bool) {
        let body = match copied {
            true => format!("{} (também na área de transferência)", path.display()),
            false => path.display().to_string(),
        };
        let icon = path.to_string_lossy();
        notify(&["-i", icon.as_ref(), "Captura salva", &body]);
    }

    fn failed(&self, message: &str) {
        notify(&["-u", "critical", "A captura falhou", message]);
    }
}

pub fn screenshot_dir() -> Result<PathBuf, String> {
    let user_dirs = UserDirs::from_env().ok_or("sem HOME")?;
    let directory = user_dirs.get(UserDir::Pictures).join(FOLDER);
    fs::create_dir_all(&directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    Ok(directory)
}

fn notify(arguments: &[&str]) {
    let result = Command::new("notify-send").args(["-a", APP_NAME]).args(arguments).status();
    if let Err(error) = result {
        eprintln!("sparky-shot: sem notificação: {error}");
    }
}
