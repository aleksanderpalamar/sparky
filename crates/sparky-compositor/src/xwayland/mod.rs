mod handler;
mod selection;

use std::os::unix::net::UnixStream;
use std::process::Stdio;

use smithay::reexports::wayland_server::Client;
use smithay::utils::{Point, Size};
use smithay::xwayland::{X11Wm, XWayland, XWaylandEvent};

use crate::cursor::CursorPixels;
use crate::state::Sparky;

impl Sparky {
    pub fn start_xwayland(&mut self) {
        let spawned = XWayland::spawn(
            &self.display_handle,
            None,
            std::iter::empty::<(String, String)>(),
            true,
            Stdio::null(),
            Stdio::null(),
            |_| (),
        );
        let (xwayland, client) = match spawned {
            Ok(pair) => pair,
            Err(error) => {
                tracing::warn!("o XWayland não subiu: {error}");
                return;
            }
        };
        self.x_display = Some(xwayland.display_number());
        let inserted = self.loop_handle.insert_source(xwayland, move |event, _, state| match event {
            XWaylandEvent::Ready { x11_socket, display_number } => {
                state.xwayland_ready(x11_socket, display_number, client.clone());
            }
            XWaylandEvent::Error => {
                tracing::warn!("o XWayland caiu ao iniciar");
                state.x_display = None;
            }
        });
        match inserted {
            Ok(token) => self.xwayland_source = Some(token),
            Err(error) => tracing::warn!("o XWayland ficou sem fonte de eventos: {error}"),
        }
    }

    pub fn stop_xwayland(&mut self) {
        self.xwm = None;
        if let Some(token) = self.xwayland_source.take() {
            self.loop_handle.remove(token);
        }
    }

    fn xwayland_ready(&mut self, socket: UnixStream, number: u32, client: Client) {
        let mut wm = match X11Wm::start_wm(self.loop_handle.clone(), socket, client) {
            Ok(wm) => wm,
            Err(error) => {
                tracing::warn!("o gerenciador de janelas X11 não subiu: {error}");
                return;
            }
        };
        let cursor = CursorPixels::load();
        let size = Size::from((cursor.width as u16, cursor.height as u16));
        let hotspot = Point::from((cursor.hotspot.0 as u16, cursor.hotspot.1 as u16));
        if let Err(error) = wm.set_cursor(&cursor.rgba, size, hotspot) {
            tracing::warn!("cursor das janelas X11 não definido: {error}");
        }
        tracing::info!("XWayland pronto em :{number}");
        self.xwm = Some(wm);
    }
}
