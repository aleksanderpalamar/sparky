use sparky_ipc::{Client, OutputInfo, Reply, Request};

use crate::ports::{Compositor, CompositorError};

pub struct SparkyIpc;

impl Compositor for SparkyIpc {
    fn reload(&self) -> Result<(), CompositorError> {
        match ask(Request::ReloadConfig)? {
            Reply::Done => Ok(()),
            Reply::Error { message } => Err(CompositorError::Refused(message)),
            other => Err(CompositorError::Refused(format!("{other:?}"))),
        }
    }

    fn outputs(&self) -> Result<Vec<OutputInfo>, CompositorError> {
        match ask(Request::Outputs)? {
            Reply::Outputs { outputs } => Ok(outputs),
            Reply::Error { message } => Err(CompositorError::Refused(message)),
            other => Err(CompositorError::Refused(format!("{other:?}"))),
        }
    }
}

fn ask(request: Request) -> Result<Reply, CompositorError> {
    let unreachable = |error: sparky_ipc::IpcError| CompositorError::Unreachable(error.to_string());
    let mut client = Client::from_env().map_err(unreachable)?;
    client.request(&request).map_err(unreachable)
}
