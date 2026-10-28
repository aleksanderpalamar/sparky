use crate::domain::grid::Spot;
use crate::domain::input::{Wheel, wheel_steps};

const MAX_STEPS: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Sgr,
    Utf8,
    Legacy,
}

pub fn wheel(lines: f32, rest: f32, at: Spot, encoding: Encoding) -> (Wheel, f32) {
    let (steps, rest) = wheel_steps(lines, rest);
    let count = (steps.abs() as usize).min(MAX_STEPS);
    let bytes = packet(steps > 0.0, at, encoding)
        .map(|packet| packet.repeat(count))
        .unwrap_or_default();
    (Wheel::Keys(bytes), rest)
}

fn packet(up: bool, at: Spot, encoding: Encoding) -> Option<Vec<u8>> {
    let button = match up {
        true => 64,
        false => 65,
    };
    let x = at.column.checked_add(1)?;
    let y = at.line.checked_add(1)?;
    match encoding {
        Encoding::Sgr => Some(format!("\x1b[<{button};{x};{y}M").into_bytes()),
        Encoding::Utf8 => extended(button, x, y),
        Encoding::Legacy => legacy(button, x, y),
    }
}

fn legacy(button: u8, x: usize, y: usize) -> Option<Vec<u8>> {
    let x = u8::try_from(x.checked_add(32)?).ok()?;
    let y = u8::try_from(y.checked_add(32)?).ok()?;
    Some(vec![0x1b, b'[', b'M', button + 32, x, y])
}

fn extended(button: u8, x: usize, y: usize) -> Option<Vec<u8>> {
    if x > 2015 || y > 2015 {
        return None;
    }
    let x = char::from_u32(u32::try_from(x + 32).ok()?)?;
    let y = char::from_u32(u32::try_from(y + 32).ok()?)?;
    Some(format!("\x1b[M{}{x}{y}", char::from(button + 32)).into_bytes())
}

#[cfg(test)]
#[path = "mouse_tests.rs"]
mod tests;
