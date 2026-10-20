use std::ffi::OsString;
use std::sync::Arc;

use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{Interest, LoopHandle, Mode, PostAction};
use smithay::reexports::wayland_server::Display;
use smithay::wayland::socket::ListeningSocketSource;

use crate::state::Sparky;
use crate::state::client::ClientState;

pub fn listen(display: Display<Sparky>, handle: &LoopHandle<'static, Sparky>) -> Result<OsString, Box<dyn std::error::Error>> {
    let socket = ListeningSocketSource::new_auto()?;
    let name = socket.socket_name().to_os_string();
    handle.insert_source(socket, |stream, _, state| {
        if let Err(error) = state.display_handle.insert_client(stream, Arc::new(ClientState::default())) {
            tracing::warn!("cliente recusado: {error}");
        }
    })?;
    handle.insert_source(Generic::new(display, Interest::READ, Mode::Level), |_, display, state| {
        let dispatched = unsafe { display.get_mut().dispatch_clients(state) };
        if let Err(error) = dispatched {
            tracing::warn!("falha ao atender os clientes: {error}");
        }
        Ok(PostAction::Continue)
    })?;
    Ok(name)
}
