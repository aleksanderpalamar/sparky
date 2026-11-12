use sparky_ipc::{Event, IdleHold, Reply, Request, WorkspaceInfo};
use sparky_tiling::WorkspaceId;

use crate::ipc::connection::write_reply;
use crate::state::Sparky;

impl Sparky {
    pub fn answer(&mut self, request: Request, hold: &mut IdleHold) -> Reply {
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
            Request::Outputs => Reply::Outputs {
                outputs: self.output_infos(),
            },
            Request::Capture { area, path } => {
                match self.capture(&area, std::path::Path::new(&path)) {
                    Ok(()) => Reply::Done,
                    Err(error) => Reply::Error {
                        message: error.to_string(),
                    },
                }
            }
            Request::Session { action } => self.session_request(action),
            Request::Drag { uris } => self.start_file_drag(uris),
            Request::Info => Reply::Info {
                info: self.session_info(),
            },
            Request::Idle { hold: wanted } => {
                self.idle.hold(*hold, wanted);
                *hold = wanted;
                Reply::Done
            }
            Request::ReloadConfig => match self.reload_config() {
                Ok(()) => Reply::Done,
                Err(error) => Reply::Error {
                    message: error.to_string(),
                },
            },
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
        self.broadcast(Event::WorkspacesChanged);
    }

    pub fn broadcast(&mut self, event: Event) {
        let Some(server) = self.ipc.as_mut() else {
            return;
        };
        let event = Reply::Event { event };
        server
            .subscribers
            .retain(|subscriber| write_reply(subscriber, &event));
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
