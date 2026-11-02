use smithay::desktop::{WindowSurfaceType, find_popup_root_surface, layer_map_for_output};
use smithay::output::Output;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::state::Sparky;
use crate::windows::surface::{root_of, root_surface};

impl Sparky {
    pub fn surface_output(&self, surface: &WlSurface) -> Option<Output> {
        let owner = self.owner(surface);
        self.window_output(&owner)
            .or_else(|| self.layer_output(&owner))
            .or_else(|| self.space.outputs().next().cloned())
    }

    fn owner(&self, surface: &WlSurface) -> WlSurface {
        let root = root_of(surface);
        self.popups
            .find_popup(&root)
            .and_then(|popup| find_popup_root_surface(&popup).ok())
            .map_or(root, |parent| root_of(&parent))
    }

    fn window_output(&self, root: &WlSurface) -> Option<Output> {
        let (_, window) = self
            .windows
            .find(|window| root_surface(window).as_ref() == Some(root))?;
        self.space.outputs_for_element(window).into_iter().next()
    }

    fn layer_output(&self, root: &WlSurface) -> Option<Output> {
        self.space
            .outputs()
            .find(|output| {
                layer_map_for_output(output)
                    .layer_for_surface(root, WindowSurfaceType::TOPLEVEL)
                    .is_some()
            })
            .cloned()
    }
}
