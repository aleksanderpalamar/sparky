use iced::widget::svg::Handle;

use crate::domain::form::Reveal;
use crate::domain::power::Power;

const USER: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="7.5" r="4.5" fill="#000"/><path d="M3 21c0-4.4 4-7.5 9-7.5s9 3.1 9 7.5z" fill="#000"/></svg>"##;
const EYE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#000" fill-rule="evenodd" d="M12 5C6.5 5 2.7 8.6 1 12c1.7 3.4 5.5 7 11 7s9.3-3.6 11-7c-1.7-3.4-5.5-7-11-7zM12 8.8a3.2 3.2 0 1 0 0 6.4a3.2 3.2 0 1 0 0-6.4z"/></svg>"##;
const EYE_OFF: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="#000" fill-rule="evenodd" d="M12 5C6.5 5 2.7 8.6 1 12c1.7 3.4 5.5 7 11 7s9.3-3.6 11-7c-1.7-3.4-5.5-7-11-7zM12 8.8a3.2 3.2 0 1 0 0 6.4a3.2 3.2 0 1 0 0-6.4z"/><path d="M3 3l18 18" stroke="#000" stroke-width="2.2" stroke-linecap="round"/></svg>"##;
const SHUTDOWN: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="1.3" stroke-linecap="round"><path d="M7.4 5.1a8.6 8.6 0 1 0 9.2 0"/><path d="M12 2.6v8.2"/></svg>"##;
const REBOOT: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M20.6 12a8.6 8.6 0 1 1-2.5-6.1"/><path d="M18.4 2.4v3.8h-3.8"/><path d="M8.6 8.6l6.8 6.8"/></svg>"##;
const SUSPEND: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="1.3" stroke-linecap="round"><circle cx="12" cy="12" r="9.4"/><path d="M10 8.6v6.8M14 8.6v6.8"/></svg>"##;

pub fn user() -> Handle {
    Handle::from_memory(USER.as_bytes())
}

pub fn password(reveal: Reveal) -> Handle {
    let source = match reveal {
        Reveal::Hidden => EYE_OFF,
        Reveal::Shown => EYE,
    };
    Handle::from_memory(source.as_bytes())
}

pub fn power(action: Power) -> Handle {
    let source = match action {
        Power::Shutdown => SHUTDOWN,
        Power::Reboot => REBOOT,
        Power::Suspend => SUSPEND,
    };
    Handle::from_memory(source.as_bytes())
}
