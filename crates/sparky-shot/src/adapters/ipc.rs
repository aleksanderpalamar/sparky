use std::path::Path;

use sparky_ipc::{CaptureArea, Client, OutputInfo, Reply, Request};

use crate::ports::Capturer;

pub struct SparkyCapturer;

impl Capturer for SparkyCapturer {
    fn capture(&self, area: &CaptureArea, path: &Path) -> Result<(), String> {
        let request = Request::Capture {
            area: area.clone(),
            path: path.to_string_lossy().into_owned(),
        };
        match ask(request)? {
            Reply::Done => Ok(()),
            Reply::Error { message } => Err(message),
            other => Err(format!("resposta inesperada do sparky: {other:?}")),
        }
    }
}

pub fn focused_output() -> Option<OutputInfo> {
    match ask(Request::Outputs).ok()? {
        Reply::Outputs { outputs } => outputs.into_iter().find(|output| output.focused),
        _ => None,
    }
}

fn ask(request: Request) -> Result<Reply, String> {
    let mut client = Client::from_env().map_err(|error| error.to_string())?;
    client.request(&request).map_err(|error| error.to_string())
}
