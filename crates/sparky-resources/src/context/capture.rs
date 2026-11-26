#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unavailable {
    Disabled,
    Unreachable,
    Unauthorized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sensed {
    Idle,
    Capturing,
    Unavailable(Unavailable),
}
