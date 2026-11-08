use smithay::desktop::layer_map_for_output;
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::shell::wlr_layer::{KeyboardInteractivity, Layer};

use crate::focus::KeyboardFocus;
use crate::state::Sparky;
use crate::windows::surface::focus_target;

impl Sparky {
    pub fn update_keyboard_focus(&mut self) {
        let target = match self.lock.is_locked() {
            true => self.lock_surface(),
            false => self.exclusive_layer().or_else(|| self.focused_surface()),
        };
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        if keyboard.current_focus() != target {
            keyboard.set_focus(self, target, SERIAL_COUNTER.next_serial());
        }
    }

    fn lock_surface(&self) -> Option<KeyboardFocus> {
        let focused = self
            .desktop
            .focused_monitor()
            .map(|monitor| monitor.name.clone());
        let output = focused
            .into_iter()
            .chain(self.space.outputs().map(|output| output.name()));
        output
            .filter_map(|name| self.lock.surface_on(&name))
            .map(|surface| KeyboardFocus::Surface(surface.wl_surface().clone()))
            .next()
    }

    fn focused_surface(&self) -> Option<KeyboardFocus> {
        let id = self.desktop.focused_window()?;
        self.windows.get(id).map(focus_target)
    }

    fn exclusive_layer(&self) -> Option<KeyboardFocus> {
        let focused = self
            .desktop
            .focused_monitor()
            .map(|monitor| monitor.name.clone());
        let mut outputs: Vec<_> = self.space.outputs().cloned().collect();
        outputs.sort_by_key(|output| Some(output.name()) != focused);
        outputs.iter().find_map(|output| {
            let map = layer_map_for_output(output);
            let layers = map
                .layers()
                .map(|layer| {
                    let state = layer.cached_state();
                    (
                        state.layer,
                        state.keyboard_interactivity,
                        layer.wl_surface().clone(),
                    )
                })
                .collect();
            newest_exclusive(layers).map(KeyboardFocus::Surface)
        })
    }
}

fn newest_exclusive<T>(layers: Vec<(Layer, KeyboardInteractivity, T)>) -> Option<T> {
    layers
        .into_iter()
        .enumerate()
        .filter(|(_, (_, interactivity, _))| *interactivity == KeyboardInteractivity::Exclusive)
        .filter_map(|(order, (layer, _, item))| Some(((rank(layer)?, order), item)))
        .max_by_key(|(key, _)| *key)
        .map(|(_, item)| item)
}

fn rank(layer: Layer) -> Option<u8> {
    match layer {
        Layer::Overlay => Some(1),
        Layer::Top => Some(0),
        Layer::Background | Layer::Bottom => None,
    }
}

#[cfg(test)]
mod tests {
    use smithay::wayland::shell::wlr_layer::{KeyboardInteractivity, Layer};

    use super::newest_exclusive;

    const EXCLUSIVE: KeyboardInteractivity = KeyboardInteractivity::Exclusive;

    #[test]
    fn a_camada_exclusiva_mais_nova_recebe_o_teclado() {
        let layers = vec![
            (Layer::Overlay, EXCLUSIVE, "painel"),
            (Layer::Overlay, EXCLUSIVE, "captura"),
        ];
        assert_eq!(newest_exclusive(layers), Some("captura"));
    }

    #[test]
    fn overlay_vence_top_e_camadas_sem_exclusividade_sao_ignoradas() {
        let layers = vec![
            (Layer::Overlay, EXCLUSIVE, "painel"),
            (Layer::Top, EXCLUSIVE, "barra"),
            (Layer::Overlay, KeyboardInteractivity::OnDemand, "aviso"),
            (Layer::Background, EXCLUSIVE, "fundo"),
        ];
        assert_eq!(newest_exclusive(layers), Some("painel"));
        let layers = vec![
            (Layer::Top, EXCLUSIVE, "barra"),
            (Layer::Bottom, EXCLUSIVE, "dock"),
        ];
        assert_eq!(newest_exclusive(layers), Some("barra"));
        assert_eq!(
            newest_exclusive(vec![(Layer::Top, KeyboardInteractivity::None, "x")]),
            None
        );
    }
}
