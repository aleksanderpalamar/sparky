use std::fs::File;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::domain::image::ImageMime;
use crate::ports::ImageClipboard;

pub struct WlImageClipboard;

impl ImageClipboard for WlImageClipboard {
    fn publish(&self, path: &Path, mime: ImageMime) -> io::Result<()> {
        let image = File::open(path)?;
        let status = Command::new("wl-copy")
            .args(["--type", mime.as_str()])
            .stdin(image)
            .stdout(Stdio::null())
            .status()?;
        if status.success() {
            return Ok(());
        }
        Err(io::Error::other(format!("wl-copy terminou com {status}")))
    }
}
