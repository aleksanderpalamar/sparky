use super::{message::Message, state::Bar};
use crate::domain::panel::PanelKind;
use iced::{Task, window};

pub fn size(width: u32, height: u32) -> (u32, u32) {
    (
        760.min(width.saturating_sub(20)).max(1),
        520.min(height.saturating_sub(58)).max(1),
    )
}

pub fn query() -> (u32, u32) {
    let Ok(mut client) = sparky_ipc::Client::from_env() else {
        return (760, 520);
    };
    let Ok(sparky_ipc::Reply::Outputs { outputs }) = client.request(&sparky_ipc::Request::Outputs)
    else {
        return (760, 520);
    };
    outputs
        .iter()
        .find(|output| output.focused)
        .or_else(|| outputs.first())
        .map_or((760, 520), |output| {
            size(output.width.max(1) as u32, output.height.max(1) as u32)
        })
}

pub fn apply(bar: &Bar, id: window::Id, size: (u32, u32)) -> Task<Message> {
    if !bar
        .panels
        .open
        .is_some_and(|open| open.window == id && open.kind == PanelKind::Clock)
    {
        return Task::none();
    }
    Task::done(Message::SizeChange { id, size })
}
