use std::time::Duration;

use iced::{Subscription, time};
use sparky_theme::Palette;

pub fn subscription() -> Subscription<Palette> {
    time::every(Duration::from_secs(1)).map(|_| sparky_theme::current_palette())
}
