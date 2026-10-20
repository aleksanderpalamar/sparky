use std::sync::OnceLock;

use smithay::reexports::wayland_server::Client;
use smithay::reexports::wayland_server::backend::{ClientData, ClientId, DisconnectReason};
use smithay::wayland::compositor::CompositorClientState;
use smithay::xwayland::XWaylandClientData;

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

pub fn compositor_state_of(client: &Client) -> &CompositorClientState {
    if let Some(data) = client.get_data::<XWaylandClientData>() {
        return &data.compositor_state;
    }
    match client.get_data::<ClientState>() {
        Some(data) => &data.compositor_state,
        None => unknown_client_state(),
    }
}

fn unknown_client_state() -> &'static CompositorClientState {
    static FALLBACK: OnceLock<CompositorClientState> = OnceLock::new();
    FALLBACK.get_or_init(CompositorClientState::default)
}
