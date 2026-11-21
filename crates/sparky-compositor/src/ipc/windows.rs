use std::os::unix::net::UnixStream;
use std::time::Duration;

use smithay::desktop::{Window, WindowSurface};
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::reexports::wayland_server::Resource;
use sparky_ipc::{Event, Reply, WindowInfo};
use sparky_tiling::{Fullscreen, Layer, Placement};

use crate::ipc::connection::write_reply;
use crate::state::Sparky;
use crate::windows::traits::traits_of;

const REFRESH: Duration = Duration::from_millis(500);

impl Sparky {
    pub fn window_infos(&self) -> Vec<WindowInfo> {
        let focused = self.desktop.focused_window();
        let placed: Vec<Placement> = self
            .desktop
            .arrange()
            .into_iter()
            .flat_map(|(_, placements)| placements)
            .collect();
        self.windows
            .iter()
            .map(|(id, window)| {
                let placement = placed.iter().find(|placement| placement.window == id);
                WindowInfo {
                    app_id: traits_of(window).app_id,
                    pid: self.pid_of(window),
                    workspace: self.desktop.workspace_of(id).map(|workspace| workspace.0),
                    visible: placement.is_some(),
                    focused: focused == Some(id),
                    fullscreen: placement.is_some_and(|placement| {
                        placement.layer == Layer::Fullscreen(Fullscreen::Monitor)
                    }),
                }
            })
            .collect()
    }

    pub fn watch_windows(&mut self, stream: UnixStream) {
        let Some(server) = self.ipc.as_mut() else {
            return;
        };
        server.window_watchers.push(stream);
        if server.window_timer.is_some() {
            return;
        }
        let inserted = self.loop_handle.insert_source(
            Timer::from_duration(REFRESH),
            |_, _, state| match state.broadcast_windows() {
                true => TimeoutAction::ToDuration(REFRESH),
                false => TimeoutAction::Drop,
            },
        );
        match inserted {
            Ok(token) => {
                if let Some(server) = self.ipc.as_mut() {
                    server.window_timer = Some(token);
                }
            }
            Err(error) => tracing::warn!("sem aviso de mudanças nas janelas: {error}"),
        }
    }

    fn broadcast_windows(&mut self) -> bool {
        let current = self.window_infos();
        let Some(server) = self.ipc.as_mut() else {
            return false;
        };
        if server.last_windows != current {
            server.last_windows = current;
            let event = Reply::Event {
                event: Event::WindowsChanged,
            };
            server
                .window_watchers
                .retain(|watcher| write_reply(watcher, &event));
        }
        if server.window_watchers.is_empty() {
            server.window_timer = None;
            return false;
        }
        true
    }

    fn pid_of(&self, window: &Window) -> Option<u32> {
        match window.underlying_surface() {
            WindowSurface::Wayland(toplevel) => {
                let client = toplevel.wl_surface().client()?;
                let credentials = client.get_credentials(&self.display_handle).ok()?;
                u32::try_from(credentials.pid).ok()
            }
            WindowSurface::X11(x11) => x11.pid(),
        }
    }
}
