use std::time::Duration;

use sparky_ipc::{Client, OutputInfo, Reply, Request, socket};

const TIMEOUT: Duration = Duration::from_secs(2);

pub fn focused_width() -> Option<u32> {
    let path = socket::from_env()?;
    let mut client = Client::connect_with_timeout(&path, TIMEOUT).ok()?;
    match client.request(&Request::Outputs).ok()? {
        Reply::Outputs { outputs } => width(&outputs),
        _ => None,
    }
}

pub fn width(outputs: &[OutputInfo]) -> Option<u32> {
    outputs
        .iter()
        .filter(|output| output.enabled && output.width > 0)
        .find(|output| output.focused)
        .or_else(|| {
            outputs
                .iter()
                .find(|output| output.enabled && output.width > 0)
        })
        .and_then(|output| u32::try_from(output.width).ok())
}
