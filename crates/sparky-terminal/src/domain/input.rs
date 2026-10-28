const START: &str = "\x1b[200~";
const END: &str = "\x1b[201~";
const UP: &[u8] = b"\x1b[A";
const DOWN: &[u8] = b"\x1b[B";
const FOCUS_IN: &[u8] = b"\x1b[I";
const FOCUS_OUT: &[u8] = b"\x1b[O";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Gained,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reporting {
    On,
    Off,
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

pub fn wheel(lines: f32, rest: f32, buffer: Buffer) -> (Wheel, f32) {
    let (steps, rest) = wheel_steps(lines, rest);
    let wheel = match buffer {
        Buffer::Main => Wheel::History(steps as i32),
        Buffer::Alternate => {
            let arrow = match steps > 0.0 {
                true => UP,
                false => DOWN,
            };
            Wheel::Keys(arrow.repeat(steps.abs() as usize))
        }
    };
    (wheel, rest)
}

pub fn wheel_steps(lines: f32, rest: f32) -> (f32, f32) {
    let total = rest + lines;
    let steps = total.trunc();
    (steps, total - steps)
}

pub fn focus_report(focus: Focus, reporting: Reporting) -> Option<&'static [u8]> {
    match (reporting, focus) {
        (Reporting::Off, _) => None,
        (Reporting::On, Focus::Gained) => Some(FOCUS_IN),
        (Reporting::On, Focus::Lost) => Some(FOCUS_OUT),
    }
}
