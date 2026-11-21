use super::app::App;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    pub app_id: String,
    pub focused: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    Closed,
    Running,
    Focused,
}

pub fn presence(app: &App, windows: &[Seen]) -> Presence {
    let mut owned = windows
        .iter()
        .filter(|seen| app.owns(&seen.app_id))
        .peekable();
    if owned.peek().is_none() {
        return Presence::Closed;
    }
    match owned.any(|seen| seen.focused) {
        true => Presence::Focused,
        false => Presence::Running,
    }
}
