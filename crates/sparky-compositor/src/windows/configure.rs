use smithay::desktop::{Window, WindowSurface};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::State;
use smithay::utils::Rectangle;
use smithay::wayland::shell::xdg::{ToplevelState, ToplevelSurface};
use smithay::xwayland::X11Surface;
use sparky_tiling::{Fullscreen, Layer, Placement, Rect};

const TILED: [State; 4] = [State::TiledLeft, State::TiledRight, State::TiledTop, State::TiledBottom];

pub fn configure(window: &Window, placement: &Placement, content: Rect) {
    window.set_activated(placement.focused);
    match window.underlying_surface() {
        WindowSurface::Wayland(toplevel) => configure_wayland(toplevel, placement, content),
        WindowSurface::X11(x11) => configure_x11(x11, placement, content),
    }
}

fn configure_wayland(toplevel: &ToplevelSurface, placement: &Placement, content: Rect) {
    toplevel.with_pending_state(|state| {
        state.size = Some((content.width, content.height).into());
        mark(state, State::Fullscreen, placement.layer == Layer::Fullscreen(Fullscreen::Monitor));
        mark(state, State::Maximized, placement.layer == Layer::Fullscreen(Fullscreen::Maximized));
        for tiled in TILED {
            mark(state, tiled, placement.layer == Layer::Tiled);
        }
    });
    match toplevel.is_initial_configure_sent() {
        true => {
            toplevel.send_pending_configure();
        }
        false => {
            toplevel.send_configure();
        }
    }
}

fn configure_x11(x11: &X11Surface, placement: &Placement, content: Rect) {
    let results = [
        x11.set_activated(placement.focused),
        x11.set_fullscreen(placement.layer == Layer::Fullscreen(Fullscreen::Monitor)),
        x11.set_maximized(placement.layer == Layer::Fullscreen(Fullscreen::Maximized)),
    ];
    let geometry = Rectangle::new((content.x, content.y).into(), (content.width, content.height).into());
    let configured = x11.configure(geometry);
    if results.iter().any(Result::is_err) || configured.is_err() {
        tracing::warn!("janela X11 {} não aceitou a configuração", x11.window_id());
    }
}

fn mark(state: &mut ToplevelState, flag: State, on: bool) {
    match on {
        true => state.states.set(flag),
        false => state.states.unset(flag),
    };
}
