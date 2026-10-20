use sparky_ipc::{Event, Reply, Request, WorkspaceInfo};
use sparky_tiling::WorkspaceId;

use crate::ipc::connection::write_reply;
use crate::state::Sparky;

impl Sparky {
    pub fn answer(&mut self, request: Request) -> Reply {
        match request {
            Request::Workspaces => Reply::Workspaces {
                workspaces: self.workspace_infos(),
            },
            Request::FocusWorkspace { id } => {
                self.desktop.focus_workspace(WorkspaceId(id));
                self.relayout();
                Reply::Done
            }
            Request::Subscribe => Reply::Done,
        }
    }

    pub fn broadcast_changes(&mut self) {
        let current = self.workspace_infos();
        let Some(server) = self.ipc.as_mut() else {
            return;
        };
        if server.last == current {
            return;
        }
        server.last = current;
        let event = Reply::Event {
            event: Event::WorkspacesChanged,
        };
        server.subscribers.retain(|subscriber| write_reply(subscriber, &event));
    }

    fn workspace_infos(&self) -> Vec<WorkspaceInfo> {
        self.desktop
            .summaries()
            .into_iter()
            .map(|summary| WorkspaceInfo {
                id: summary.id.0,
                monitor: summary.monitor,
                windows: summary.windows,
                visible: summary.visible,
                focused: summary.focused,
            })
            .collect()
    }
}
