use std::path::Path;

use sparky_ipc::CaptureArea;

pub trait Capturer {
    fn capture(&self, area: &CaptureArea, path: &Path) -> Result<(), String>;
}

pub trait Clipboard {
    fn copy_png(&self, path: &Path) -> Result<(), String>;
}

pub trait Notifier {
    fn saved(&self, path: &Path, copied: bool);

    fn failed(&self, message: &str);
}
