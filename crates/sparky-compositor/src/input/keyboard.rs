use smithay::backend::input::{Event, InputBackend, KeyState, KeyboardKeyEvent};
use smithay::input::keyboard::{FilterResult, KeysymHandle, ModifiersState};
use smithay::utils::SERIAL_COUNTER;

use crate::backend::drm::vt;
use sparky_config::Action;
use crate::bindings::mods_of;
use crate::state::Sparky;

impl Sparky {
    pub fn on_key<I: InputBackend>(&mut self, event: I::KeyboardKeyEvent) {
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        let code = event.key_code();
        let pressed = event.state();
        let serial = SERIAL_COUNTER.next_serial();
        let time = Event::time_msec(&event);
        let decision = keyboard.input(self, code, pressed, serial, time, |state, modifiers, handle| {
            let raw = handle.raw_code().raw();
            match pressed {
                KeyState::Pressed => state.press(raw, modifiers, &handle),
                KeyState::Released => state.release(raw),
            }
        });
        if let Some(Some(action)) = decision {
            self.run(action);
        }
    }

    fn press(&mut self, code: u32, modifiers: &ModifiersState, handle: &KeysymHandle<'_>) -> FilterResult<Option<Action>> {
        let terminal = vt::terminal_for(handle.modified_sym().raw()).map(|terminal| Action::SwitchTerminal { terminal });
        let sym = handle.raw_latin_sym_or_raw_current_sym();
        let action = terminal.or_else(|| self.bindings.find(mods_of(modifiers), code, sym).cloned());
        match action {
            Some(action) => {
                self.suppressed_keys.push(code);
                FilterResult::Intercept(Some(action))
            }
            None => FilterResult::Forward,
        }
    }

    fn release(&mut self, code: u32) -> FilterResult<Option<Action>> {
        let before = self.suppressed_keys.len();
        self.suppressed_keys.retain(|known| *known != code);
        match before == self.suppressed_keys.len() {
            true => FilterResult::Forward,
            false => FilterResult::Intercept(None),
        }
    }
}
