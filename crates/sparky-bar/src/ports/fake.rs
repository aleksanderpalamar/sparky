use std::sync::Mutex;

use sparky_ipc::{IpcError, WorkspaceInfo};

use crate::ports::workspaces::Workspaces;

#[derive(Default)]
pub struct FakeWorkspaces {
    listed: Option<Vec<WorkspaceInfo>>,
    focused: Mutex<Vec<u32>>,
}

impl FakeWorkspaces {
    pub fn with(workspaces: Vec<WorkspaceInfo>) -> Self {
        Self {
            listed: Some(workspaces),
            focused: Mutex::default(),
        }
    }

    pub fn focused(&self) -> Vec<u32> {
        match self.focused.lock() {
            Ok(focused) => focused.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

impl Workspaces for FakeWorkspaces {
    fn list(&self) -> Result<Vec<WorkspaceInfo>, IpcError> {
        self.listed.clone().ok_or(IpcError::Unavailable)
    }

    fn focus(&self, id: u32) -> Result<(), IpcError> {
        match self.focused.lock() {
            Ok(mut focused) => focused.push(id),
            Err(poisoned) => poisoned.into_inner().push(id),
        }
        Ok(())
    }
}
