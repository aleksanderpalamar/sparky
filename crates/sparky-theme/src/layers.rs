use crate::palette::Mode;

pub const WINDOW: f32 = 0.76;

pub fn card(mode: Mode, window: f32) -> f32 {
    let lift = match mode {
        Mode::Dark => 0.30,
        Mode::Light => 0.25,
    };
    (window + lift).min(1.0)
}

pub fn spread(window: f32) -> f32 {
    (1.0 - window).clamp(0.0, 1.0)
}
