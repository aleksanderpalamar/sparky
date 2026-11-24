use smithay::desktop::{PopupKind, PopupManager, find_popup_root_surface};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::IsAlive;

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

impl Sparky {
    pub fn hold_popup(&mut self, popup: PopupKind) {
        let Ok(root) = find_popup_root_surface(&popup) else {
            return;
        };
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
        if let Err(error) = PopupManager::dismiss_popup(&hold.root, &hold.popup) {
            tracing::debug!("menu já tinha fechado: {error:?}");
        }
        self.popup_hold = None;
        self.update_keyboard_focus();
        true
    }
}
