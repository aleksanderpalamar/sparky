use smithay::desktop::layer_map_for_output;
use smithay::output::{Output, Scale};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::Transform;
use smithay::wayland::compositor::{send_surface_state, with_states};
use smithay::wayland::fractional_scale::with_fractional_scale;
use smithay::wayland::seat::WaylandFocus;
use sparky_config::scale_for;

use crate::state::Sparky;

pub fn row(widths: &[i32]) -> Vec<i32> {
    widths
        .iter()
        .scan(0, |next, width| {
            let x = *next;
            *next += width;
            Some(x)
        })
        .collect()
}

pub fn preferred(scales: impl IntoIterator<Item = f64>) -> f64 {
    scales.into_iter().fold(None, |best: Option<f64>, scale| Some(best.map_or(scale, |best| best.max(scale)))).unwrap_or(1.0)
}

impl Sparky {
    pub fn configured_scale(&self, name: &str) -> Scale {
        Scale::Fractional(scale_for(&self.config.output, name).factor())
    }

    pub fn apply_output_scales(&mut self) {
        let mut outputs: Vec<Output> = self.space.outputs().cloned().collect();
        outputs.sort_by_key(|output| self.space.output_geometry(output).map(|geometry| geometry.loc.x));
        for output in &outputs {
            output.change_current_state(None, None, Some(self.configured_scale(&output.name())), None);
        }
        let widths: Vec<i32> = outputs
            .iter()
            .map(|output| self.space.output_geometry(output).map_or(0, |geometry| geometry.size.w))
            .collect();
        for (output, x) in outputs.iter().zip(row(&widths)) {
            output.change_current_state(None, None, None, Some((x, 0).into()));
            self.space.map_output(output, (x, 0));
        }
        self.sync_outputs();
        self.relayout();
    }

    pub fn publish_scales(&self) {
        for window in self.space.elements() {
            let Some(bounds) = self.space.element_bbox(window) else {
                continue;
            };
            let scale = preferred(
                self.space
                    .outputs()
                    .filter(|output| self.space.output_geometry(output).is_some_and(|screen| screen.overlaps(bounds)))
                    .map(|output| output.current_scale().fractional_scale()),
            );
            if let Some(surface) = window.wl_surface() {
                announce(&surface, scale);
            }
        }
        for output in self.space.outputs() {
            let scale = output.current_scale().fractional_scale();
            for layer in layer_map_for_output(output).layers() {
                announce(layer.wl_surface(), scale);
            }
        }
    }
}

fn announce(surface: &WlSurface, scale: f64) {
    with_states(surface, |states| {
        with_fractional_scale(states, |fractional| fractional.set_preferred_scale(scale));
        send_surface_state(surface, states, scale.ceil() as i32, Transform::Normal);
    });
}

#[cfg(test)]
mod tests {
    use super::{preferred, row};

    #[test]
    fn os_monitores_ficam_em_fila_pela_largura_logica() {
        assert_eq!(row(&[1920, 1707, 2560]), vec![0, 1920, 3627]);
        assert!(row(&[]).is_empty());
    }

    #[test]
    fn janela_em_dois_monitores_pede_a_maior_escala() {
        assert_eq!(preferred([1.0, 1.5]), 1.5);
        assert_eq!(preferred([]), 1.0);
    }
}
