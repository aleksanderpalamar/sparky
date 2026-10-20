use smithay::backend::renderer::utils::on_commit_buffer_handler;
use smithay::reexports::wayland_server::Client;
use smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState, get_parent, is_sync_subsurface};
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::{delegate_compositor, delegate_shm};

use crate::state::Sparky;
use crate::state::client::compositor_state_of;

impl CompositorHandler for Sparky {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.protocols.compositor
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        compositor_state_of(client)
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        self.needs_render = true;
        if !is_sync_subsurface(surface) {
            let root = root_of(surface);
            if let Some((_, window)) = self.windows.find(|window| window.toplevel().is_some_and(|t| t.wl_surface() == &root)) {
                window.on_commit();
            }
        }
        self.popups.commit(surface);
        self.commit_popup(surface);
        self.commit_layer(surface);
        self.commit_toplevel(surface);
    }
}

impl BufferHandler for Sparky {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}

impl ShmHandler for Sparky {
    fn shm_state(&self) -> &ShmState {
        &self.protocols.shm
    }
}

fn root_of(surface: &WlSurface) -> WlSurface {
    let mut root = surface.clone();
    while let Some(parent) = get_parent(&root) {
        root = parent;
    }
    root
}

delegate_compositor!(Sparky);
delegate_shm!(Sparky);
