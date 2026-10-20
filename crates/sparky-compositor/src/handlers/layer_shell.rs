use smithay::delegate_layer_shell;
use smithay::desktop::{LayerSurface, layer_map_for_output};
use smithay::output::Output;
use smithay::reexports::wayland_server::protocol::wl_output::WlOutput;
use smithay::wayland::shell::wlr_layer::{Layer, LayerSurface as WlrLayerSurface, WlrLayerShellHandler, WlrLayerShellState};

use crate::state::Sparky;

impl WlrLayerShellHandler for Sparky {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.protocols.layer_shell
    }

    fn new_layer_surface(&mut self, surface: WlrLayerSurface, output: Option<WlOutput>, _layer: Layer, namespace: String) {
        let Some(output) = output.as_ref().and_then(Output::from_resource).or_else(|| self.focused_output()) else {
            surface.send_close();
            return;
        };
        let mut map = layer_map_for_output(&output);
        if let Err(error) = map.map_layer(&LayerSurface::new(surface, namespace)) {
            tracing::warn!("camada recusada: {error}");
        }
    }

    fn layer_destroyed(&mut self, surface: WlrLayerSurface) {
        let outputs: Vec<Output> = self.space.outputs().cloned().collect();
        for output in outputs {
            let mut map = layer_map_for_output(&output);
            let found = map.layers().find(|layer| layer.layer_surface() == &surface).cloned();
            if let Some(layer) = found {
                map.unmap_layer(&layer);
            }
        }
        self.sync_outputs();
        self.relayout();
    }
}

impl Sparky {
    pub fn focused_output(&self) -> Option<Output> {
        let name = self.desktop.focused_monitor().map(|monitor| monitor.name.clone());
        self.space
            .outputs()
            .find(|output| Some(output.name()) == name)
            .or_else(|| self.space.outputs().next())
            .cloned()
    }
}

delegate_layer_shell!(Sparky);
