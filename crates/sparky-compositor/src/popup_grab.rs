use smithay::desktop::{PopupKind, PopupManager, find_popup_root_surface, layer_map_for_output};
use smithay::reexports::wayland_server::Resource;
use smithay::reexports::wayland_server::backend::ClientId;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::IsAlive;
use smithay::wayland::seat::WaylandFocus;

use crate::focus::KeyboardFocus;
use crate::state::Sparky;

pub struct PopupHold {
    root: WlSurface,
    popup: PopupKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Above {
    Nothing,
    ExclusiveLayer,
}

pub fn dismisses<T: PartialEq>(under: Option<&T>, tree: &[T], above: Above) -> bool {
    match above {
        Above::ExclusiveLayer => false,
        Above::Nothing => under.is_none_or(|surface| !tree.contains(surface)),
    }
}

pub fn trusted<T: PartialEq>(owner: Option<&T>, keyboard: Option<&T>, pointer: Option<&T>) -> bool {
    owner.is_some_and(|owner| keyboard == Some(owner) || pointer == Some(owner))
}

pub fn visible<T: PartialEq>(root: &T, shown: &[T]) -> bool {
    shown.contains(root)
}

impl Sparky {
    pub fn hold_popup(&mut self, popup: PopupKind) {
        let Ok(root) = find_popup_root_surface(&popup) else {
            return;
        };
        if !self.grab_allowed(&root) {
            refuse(&popup);
            return;
        }
        self.popup_hold = Some(PopupHold { root, popup });
        self.update_keyboard_focus();
    }

    pub fn held_popup(&self) -> Option<KeyboardFocus> {
        let hold = self.popup_hold.as_ref()?;
        hold.popup
            .alive()
            .then(|| KeyboardFocus::Surface(hold.popup.wl_surface().clone()))
    }

    pub fn release_popup(&mut self, surface: &WlSurface) {
        let held = self
            .popup_hold
            .as_ref()
            .is_some_and(|hold| hold.popup.wl_surface() == surface);
        if held {
            self.popup_hold = None;
            self.update_keyboard_focus();
        }
    }

    pub fn dismiss_outside(&mut self, under: Option<&WlSurface>) -> bool {
        let Some(hold) = self.popup_hold.as_ref() else {
            return false;
        };
        let tree: Vec<WlSurface> = PopupManager::popups_for_surface(&hold.root)
            .map(|(popup, _)| popup.wl_surface().clone())
            .collect();
        let above = match self.exclusive_layer() {
            Some(_) => Above::ExclusiveLayer,
            None => Above::Nothing,
        };
        if !dismisses(under, &tree, above) {
            return false;
        }
        self.drop_hold();
        self.update_keyboard_focus();
        true
    }

    pub fn dismiss_hidden(&mut self) {
        let Some(hold) = self.popup_hold.as_ref() else {
            return;
        };
        if visible(&hold.root, &self.shown_roots()) {
            return;
        }
        self.drop_hold();
    }

    fn drop_hold(&mut self) {
        let Some(hold) = self.popup_hold.take() else {
            return;
        };
        if let Err(error) = PopupManager::dismiss_popup(&hold.root, &hold.popup) {
            tracing::debug!("menu já tinha fechado: {error:?}");
        }
    }

    fn shown_roots(&self) -> Vec<WlSurface> {
        let windows = self
            .space
            .elements()
            .filter_map(|window| window.wl_surface().map(|surface| surface.into_owned()));
        let layers = self.space.outputs().flat_map(|output| {
            layer_map_for_output(output)
                .layers()
                .map(|layer| layer.wl_surface().clone())
                .collect::<Vec<_>>()
        });
        windows.chain(layers).collect()
    }

    fn grab_allowed(&self, root: &WlSurface) -> bool {
        let keyboard = self
            .seat
            .get_keyboard()
            .and_then(|keyboard| keyboard.current_focus())
            .and_then(|focus| focus.wl_surface().and_then(|surface| client_of(&surface)));
        let pointer = self
            .seat
            .get_pointer()
            .and_then(|pointer| pointer.current_focus())
            .and_then(|surface| client_of(&surface));
        trusted(
            client_of(root).as_ref(),
            keyboard.as_ref(),
            pointer.as_ref(),
        )
    }
}

fn client_of(surface: &WlSurface) -> Option<ClientId> {
    surface.client().map(|client| client.id())
}

fn refuse(popup: &PopupKind) {
    if let PopupKind::Xdg(surface) = popup {
        surface.send_popup_done();
    }
}
