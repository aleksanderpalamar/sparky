use smithay::desktop::{PopupKind, WindowSurfaceType, layer_map_for_output};
use smithay::output::Output;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::wlr_layer::LayerSurfaceData;
use smithay::wayland::shell::xdg::{SurfaceCachedState, ToplevelSurface, XdgToplevelSurfaceData};

use crate::state::{BORDER, Sparky};
use crate::windows::rules::{self, Traits};

impl Sparky {
    pub fn commit_popup(&mut self, surface: &WlSurface) {
        let Some(PopupKind::Xdg(popup)) = self.popups.find_popup(surface) else {
            return;
        };
        if !popup.is_initial_configure_sent()
            && let Err(error) = popup.send_configure()
        {
            tracing::warn!("menu sem configuração inicial: {error}");
        }
    }

    pub fn commit_layer(&mut self, surface: &WlSurface) {
        let Some(output) = self.output_with_layer(surface) else {
            return;
        };
        let initial_sent = with_states(surface, |states| {
            states
                .data_map
                .get::<LayerSurfaceData>()
                .and_then(|data| data.lock().ok().map(|data| data.initial_configure_sent))
                .unwrap_or(true)
        });
        {
            let mut map = layer_map_for_output(&output);
            map.arrange();
            if !initial_sent && let Some(layer) = map.layer_for_surface(surface, WindowSurfaceType::TOPLEVEL) {
                layer.layer_surface().send_configure();
            }
        }
        self.sync_outputs();
        self.relayout();
    }

    pub fn commit_toplevel(&mut self, surface: &WlSurface) {
        let Some((id, window)) = self
            .windows
            .find(|window| window.toplevel().is_some_and(|toplevel| toplevel.wl_surface() == surface))
        else {
            return;
        };
        if self.desktop.workspace_of(id).is_some() {
            return;
        }
        let Some(toplevel) = window.toplevel().cloned() else {
            return;
        };
        let opening = rules::opening(&traits_of(&toplevel), self.desktop.style(), BORDER);
        self.desktop.add_window(id, opening);
        self.relayout();
    }

    fn output_with_layer(&self, surface: &WlSurface) -> Option<Output> {
        self.space
            .outputs()
            .find(|output| {
                layer_map_for_output(output)
                    .layer_for_surface(surface, WindowSurfaceType::TOPLEVEL)
                    .is_some()
            })
            .cloned()
    }
}

fn traits_of(toplevel: &ToplevelSurface) -> Traits {
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
        has_parent: toplevel.parent().is_some(),
        min_size,
        max_size,
    }
}
