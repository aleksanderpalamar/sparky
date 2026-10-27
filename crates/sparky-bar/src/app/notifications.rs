use super::{message::Message, state::Bar};
use crate::domain::panel::PanelKind;
use iced::Task;
use sparky_notifications::client::{Command, Event};

pub fn event(bar: &mut Bar, event: Event) -> Task<Message> {
    match event {
        Event::Ready(client) => {
            bar.center.client = Some(client);
            bar.center.error = None;
            if bar
                .panels
                .open
                .is_some_and(|open| open.kind == PanelKind::Clock)
            {
                return lease(bar, true);
            }
        }
        Event::State(snapshot) => {
            bar.center.snapshot = snapshot;
            bar.center.error = None;
        }
        Event::Unavailable(error) => {
            bar.center.client = None;
            bar.center.error = Some(error);
        }
    }
    Task::none()
}

pub fn request(bar: &mut Bar, command: Command) -> Task<Message> {
    let Some(client) = bar.center.client.clone() else {
        return Task::none();
    };
    if bar.panels.pending.contains_key(&PanelKind::Clock) {
        return Task::none();
    }
    let id = bar.panels.begin(PanelKind::Clock);
    Task::perform(async move { client.send(command).await }, move |result| {
        Message::NotificationFinished(id, result)
    })
}

pub fn lease(bar: &Bar, open: bool) -> Task<Message> {
    let Some(client) = bar.center.client.clone() else {
        return Task::none();
    };
    client.desire_center(open);
    Task::perform(async move { client.sync_center().await }, |_| {
        Message::Ignore
    })
}
