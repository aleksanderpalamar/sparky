use sparky_resources::protocol::{Reply, Request, Status, ask, socket};

use crate::ports::resources::ResourceService;

pub struct ResourcesSocket;

impl ResourceService for ResourcesSocket {
    fn status(&self) -> Result<Status, String> {
        match call(Request::Status)? {
            Reply::Status(status) => Ok(status),
            other => Err(format!("{other:?}")),
        }
    }

    fn send(&self, request: Request) -> Result<(), String> {
        match call(request)? {
            Reply::Done => Ok(()),
            other => Err(format!("{other:?}")),
        }
    }
}

fn call(request: Request) -> Result<Reply, String> {
    let path = socket::from_env().ok_or_else(|| "XDG_RUNTIME_DIR".to_owned())?;
    ask(&path, request).map_err(|error| format!("{error:?}"))
}
