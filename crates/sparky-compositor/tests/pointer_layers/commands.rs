use smithay::backend::input::{ButtonState, InputTime};
use smithay::input::pointer::RelativeMotionEvent;
use smithay::utils::{Logical, Point};
use sparky_compositor::state::Sparky;
use sparky_tiling::Fullscreen;

const LEFT: u32 = 0x110;

pub fn run(state: &mut Sparky, command: &str) {
    let words: Vec<&str> = command.split_whitespace().collect();
    match words.as_slice() {
        ["move", x, y] => state.move_pointer(point(x, y), InputTime::now()),
        ["shift", x, y] => state.shift_pointer(RelativeMotionEvent {
            delta: point(x, y),
            delta_unaccel: point(x, y),
            time: InputTime::now(),
        }),
        ["press"] => state.click(LEFT, ButtonState::Pressed, InputTime::now()),
        ["release"] => state.click(LEFT, ButtonState::Released, InputTime::now()),
        ["fullscreen"] => toggle(state, Fullscreen::Monitor),
        ["maximize"] => toggle(state, Fullscreen::Maximized),
        ["lock"] => state.lock.engage(),
        _ => {}
    }
}

fn toggle(state: &mut Sparky, mode: Fullscreen) {
    state.desktop.toggle_fullscreen(mode);
    state.relayout();
}

fn point(x: &str, y: &str) -> Point<f64, Logical> {
    (x.parse().unwrap_or_default(), y.parse().unwrap_or_default()).into()
}
