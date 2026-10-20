use sparky_ipc::{Client, IpcError, Reply, Request, WorkspaceInfo};

pub trait Workspaces: Send + Sync {
    fn list(&self) -> Result<Vec<WorkspaceInfo>, IpcError>;

    fn focus(&self, id: u32) -> Result<(), IpcError>;
}

pub struct SparkyWorkspaces;

impl Workspaces for SparkyWorkspaces {
    fn list(&self) -> Result<Vec<WorkspaceInfo>, IpcError> {
        match Client::from_env()?.request(&Request::Workspaces)? {
            Reply::Workspaces { workspaces } => Ok(workspaces),
            other => Err(unexpected(other)),
        }
    }

    fn focus(&self, id: u32) -> Result<(), IpcError> {
        match Client::from_env()?.request(&Request::FocusWorkspace { id })? {
            Reply::Done => Ok(()),
            other => Err(unexpected(other)),
        }
    }
}

fn unexpected(reply: Reply) -> IpcError {
    match reply {
        Reply::Error { message } => IpcError::Malformed(message),
        other => IpcError::Malformed(format!("resposta inesperada: {other:?}")),
    }
}
