use std::time::Duration;

use sparky_ipc::{Client, GlassArea, Request, socket};

const TIMEOUT: Duration = Duration::from_secs(1);

pub fn area(size: (u32, u32)) -> GlassArea {
    GlassArea::Centered {
        width: i32::try_from(size.0).unwrap_or(i32::MAX),
        height: i32::try_from(size.1).unwrap_or(i32::MAX),
    }
}

pub fn announce(namespace: &'static str, size: (u32, u32)) {
    std::thread::spawn(move || {
        let Some(path) = socket::from_env() else {
            return;
        };
        let request = Request::Glass {
            namespace: namespace.to_owned(),
            area: area(size),
        };
        let sent = Client::connect_with_timeout(&path, TIMEOUT)
            .and_then(|mut client| client.request(&request));
        if let Err(error) = sent {
            eprintln!("sparky-launcher: área do vidro não informada: {error}");
        }
    });
}
