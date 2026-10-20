use sparky_ipc::{Event, OutputInfo, Reply, Request, WorkspaceInfo};
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
            Request::Outputs => Reply::Outputs {
                outputs: self.output_infos(),
            },
            Request::Capture { area, path } => match self.capture(&area, std::path::Path::new(&path)) {
                Ok(()) => Reply::Done,
                Err(error) => Reply::Error {
                    message: error.to_string(),
                },
            },
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
        let event = Reply::Event {
            event: Event::WorkspacesChanged,
        };
        server.subscribers.retain(|subscriber| write_reply(subscriber, &event));
    }

    fn output_infos(&self) -> Vec<OutputInfo> {
        let focused = self.desktop.focused_monitor().map(|monitor| monitor.name.clone());
        self.desktop
            .monitors()
            .iter()
            .map(|monitor| OutputInfo {
                name: monitor.name.clone(),
                x: monitor.screen.full.x,
                y: monitor.screen.full.y,
                width: monitor.screen.full.width,
                height: monitor.screen.full.height,
                focused: focused.as_deref() == Some(monitor.name.as_str()),
            })
            .collect()
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
