use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime};

use iced::futures::channel::mpsc::{UnboundedSender, unbounded};
use iced::futures::{SinkExt, Stream, StreamExt};
use sparky_config::Dock;

const INTERVAL: Duration = Duration::from_secs(1);

pub fn changes() -> impl Stream<Item = Dock> {
    iced::stream::channel(4, async |mut output| {
        let (sender, mut found) = unbounded();
        thread::spawn(move || follow(&sender));
        while let Some(dock) = found.next().await {
            if output.send(dock).await.is_err() {
                return;
            }
        }
    })
}

pub fn read(path: &Path) -> Result<Dock, String> {
    match fs::read_to_string(path) {
        Ok(text) => sparky_config::parse(&text)
            .map(|config| config.dock)
            .map_err(|error| error.to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Dock::default()),
        Err(error) => Err(error.to_string()),
    }
}

fn follow(sender: &UnboundedSender<Dock>) {
    let path = location();
    let mut seen: Option<Option<SystemTime>> = None;
    while !sender.is_closed() {
        let stamp = fs::metadata(&path).and_then(|meta| meta.modified()).ok();
        if seen != Some(stamp) {
            seen = Some(stamp);
            match read(&path) {
                Ok(dock) if sender.unbounded_send(dock).is_err() => return,
                Ok(_) => {}
                Err(error) => eprintln!("sparky-dock: {}: {error}", path.display()),
            }
        }
        thread::sleep(INTERVAL);
    }
}

fn location() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    sparky_config::path(std::env::var("XDG_CONFIG_HOME").ok().as_deref(), &home)
}
