use std::borrow::Cow;

use smithay::backend::input::KeyState;
use smithay::input::Seat;
use smithay::input::keyboard::{KeyboardTarget, KeysymHandle, ModifiersState};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{IsAlive, Serial};
use smithay::wayland::seat::WaylandFocus;
use smithay::xwayland::X11Surface;

use crate::state::Sparky;

#[derive(Debug, Clone, PartialEq)]
pub enum KeyboardFocus {
    Surface(WlSurface),
    X11(X11Surface),
}

impl IsAlive for KeyboardFocus {
    fn alive(&self) -> bool {
        match self {
            KeyboardFocus::Surface(surface) => surface.alive(),
            KeyboardFocus::X11(window) => window.alive(),
        }
    }
}

impl WaylandFocus for KeyboardFocus {
    fn wl_surface(&self) -> Option<Cow<'_, WlSurface>> {
        match self {
            KeyboardFocus::Surface(surface) => Some(Cow::Borrowed(surface)),
            KeyboardFocus::X11(window) => window.wl_surface().map(Cow::Owned),
        }
    }
}

impl KeyboardTarget<Sparky> for KeyboardFocus {
    fn enter(&self, seat: &Seat<Sparky>, data: &mut Sparky, keys: Vec<KeysymHandle<'_>>, serial: Serial) {
        match self {
            KeyboardFocus::Surface(surface) => KeyboardTarget::enter(surface, seat, data, keys, serial),
            KeyboardFocus::X11(window) => KeyboardTarget::enter(window, seat, data, keys, serial),
        }
    }

    fn leave(&self, seat: &Seat<Sparky>, data: &mut Sparky, serial: Serial) {
        match self {
            KeyboardFocus::Surface(surface) => KeyboardTarget::leave(surface, seat, data, serial),
            KeyboardFocus::X11(window) => KeyboardTarget::leave(window, seat, data, serial),
        }
    }

    fn key(
        &self,
        seat: &Seat<Sparky>,
        data: &mut Sparky,
        key: KeysymHandle<'_>,
        state: KeyState,
        serial: Serial,
        time: u32,
    ) {
        match self {
            KeyboardFocus::Surface(surface) => KeyboardTarget::key(surface, seat, data, key, state, serial, time),
            KeyboardFocus::X11(window) => KeyboardTarget::key(window, seat, data, key, state, serial, time),
        }
    }

    fn modifiers(&self, seat: &Seat<Sparky>, data: &mut Sparky, modifiers: ModifiersState, serial: Serial) {
        match self {
            KeyboardFocus::Surface(surface) => KeyboardTarget::modifiers(surface, seat, data, modifiers, serial),
            KeyboardFocus::X11(window) => KeyboardTarget::modifiers(window, seat, data, modifiers, serial),
        }
    }
}
