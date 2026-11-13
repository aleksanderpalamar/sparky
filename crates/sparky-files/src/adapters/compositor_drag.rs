use std::time::Duration;

use sparky_ipc::{Client, Reply, Request, socket};

use crate::i18n;
use crate::ports::Dragger;

const TIMEOUT: Duration = Duration::from_secs(2);

pub struct CompositorDrag;

impl Dragger for CompositorDrag {
    fn start(&self, uris: Vec<String>) -> Result<(), String> {
        let path = socket::from_env().ok_or_else(|| i18n::text("drag-outside-sparky"))?;
        let mut client =
            Client::connect_with_timeout(&path, TIMEOUT).map_err(|error| error.to_string())?;
        match client.request(&Request::Drag { uris }) {
            Ok(Reply::Done) => Ok(()),
            Ok(Reply::Error { message }) => Err(message),
            Ok(_) => Err(i18n::text("drag-outside-sparky")),
            Err(error) => Err(error.to_string()),
        }
    }
}
