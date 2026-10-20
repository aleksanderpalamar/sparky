use smithay::desktop::{Window, WindowSurface};
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::xdg::{SurfaceCachedState, ToplevelSurface, XdgToplevelSurfaceData};
use smithay::xwayland::X11Surface;
use smithay::xwayland::xwm::WmWindowType;

use crate::windows::rules::Traits;

pub fn traits_of(window: &Window) -> Traits {
    match window.underlying_surface() {
        WindowSurface::Wayland(toplevel) => wayland_traits(toplevel),
        WindowSurface::X11(x11) => x11_traits(x11),
    }
}

fn wayland_traits(toplevel: &ToplevelSurface) -> Traits {
    let app_id = with_states(toplevel.wl_surface(), |states| {
        states
            .data_map
            .get::<XdgToplevelSurfaceData>()
            .and_then(|data| data.lock().ok().and_then(|data| data.app_id.clone()))
            .unwrap_or_default()
    });
    let (min_size, max_size) = with_states(toplevel.wl_surface(), |states| {
        let mut cached = states.cached_state.get::<SurfaceCachedState>();
        let current = cached.current();
        (
            (current.min_size.w, current.min_size.h),
            (current.max_size.w, current.max_size.h),
        )
    });
    Traits {
        app_id,
        dialog: toplevel.parent().is_some(),
        min_size,
        max_size,
    }
}

fn x11_traits(x11: &X11Surface) -> Traits {
    let size = |hint: Option<smithay::utils::Size<i32, smithay::utils::Logical>>| {
        hint.map(|size| (size.w, size.h)).unwrap_or((0, 0))
    };
    let dialog = matches!(
        x11.window_type(),
        Some(WmWindowType::Dialog | WmWindowType::Utility | WmWindowType::Splash | WmWindowType::Toolbar)
    );
    Traits {
        app_id: x11.class(),
        dialog: dialog || x11.is_transient_for().is_some(),
        min_size: size(x11.min_size()),
        max_size: size(x11.max_size()),
    }
}
