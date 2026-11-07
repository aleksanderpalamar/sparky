use std::path::PathBuf;
use std::time::Duration;

use sparky_ipc::{Client, IpcError, OutputInfo, Reply, Request, SessionInfo};

use crate::ipc::port::{Compositor, Problem, Snapshot};

const UNEXPECTED: &str = "unexpected reply";

pub struct SocketCompositor {
    path: Option<PathBuf>,
    timeout: Duration,
}

impl SocketCompositor {
    pub fn new(path: Option<PathBuf>, timeout: Duration) -> Self {
        Self { path, timeout }
    }

    pub fn from_env(timeout: Duration) -> Self {
        Self::new(sparky_ipc::socket::from_env(), timeout)
    }
}

impl Compositor for SocketCompositor {
    fn snapshot(&self) -> Snapshot {
        let Some(path) = &self.path else {
            return Snapshot::absent(Problem::NoCompositor);
        };
        let Ok(mut client) = Client::connect_with_timeout(path, self.timeout) else {
            return Snapshot::absent(Problem::NoCompositor);
        };
        let info = info(&mut client);
        if info == Err(Problem::TimedOut) {
            return Snapshot::absent(Problem::TimedOut);
        }
        Snapshot {
            info,
            outputs: outputs(&mut client),
        }
    }
}

fn info(client: &mut Client) -> Result<SessionInfo, Problem> {
    match client.request(&Request::Info).map_err(problem)? {
        Reply::Info { info } => Ok(info),
        Reply::Error { .. } => Err(Problem::Unsupported),
        _ => Err(Problem::Failed(UNEXPECTED.to_owned())),
    }
}

fn outputs(client: &mut Client) -> Result<Vec<OutputInfo>, Problem> {
    match client.request(&Request::Outputs).map_err(problem)? {
        Reply::Outputs { outputs } => Ok(outputs),
        Reply::Error { message } => Err(Problem::Failed(message)),
        _ => Err(Problem::Failed(UNEXPECTED.to_owned())),
    }
}

fn problem(error: IpcError) -> Problem {
    match error {
        IpcError::TimedOut => Problem::TimedOut,
        IpcError::Unavailable => Problem::NoCompositor,
        other => Problem::Failed(other.to_string()),
    }
}
