const START: &str = "\x1b[200~";
const END: &str = "\x1b[201~";
const UP: &[u8] = b"\x1b[A";
const DOWN: &[u8] = b"\x1b[B";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bracketed {
    No,
    Yes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Buffer {
    Main,
    Alternate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wheel {
    History(i32),
    Keys(Vec<u8>),
}

pub fn paste(text: &str, bracketed: Bracketed) -> Vec<u8> {
    let typed = text.replace("\r\n", "\r").replace('\n', "\r");
    match bracketed {
        Bracketed::No => typed.into_bytes(),
        Bracketed::Yes => format!("{START}{}{END}", typed.replace('\x1b', "")).into_bytes(),
    }
}

pub fn wheel(lines: f32, screen: Buffer) -> Wheel {
    let steps = lines.trunc() as i32;
    match screen {
        Buffer::Main => Wheel::History(steps),
        Buffer::Alternate => {
            let arrow = match steps > 0 {
                true => UP,
                false => DOWN,
            };
            Wheel::Keys(arrow.repeat(steps.unsigned_abs() as usize))
        }
    }
}
