mod hit;
mod keyboard;
pub mod keyboard_layout;
mod pointer;

use smithay::backend::input::{InputBackend, InputEvent};

use crate::state::Sparky;

impl Sparky {
    pub fn process_input_event<I: InputBackend>(&mut self, event: InputEvent<I>) {
        match event {
            InputEvent::Keyboard { event } => self.on_key::<I>(event),
            InputEvent::PointerMotionAbsolute { event } => self.on_absolute_motion::<I>(event),
            InputEvent::PointerButton { event } => self.on_button::<I>(event),
            InputEvent::PointerAxis { event } => self.on_axis::<I>(event),
            _ => {}
        }
    }
}
