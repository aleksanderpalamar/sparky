use std::sync::atomic::Ordering;

use iced::{Task, widget::operation};

use super::{Message, Window};
use crate::domain::{Event, Outcome};
use crate::ports::Conversation;

pub fn start(window: &mut Window) -> Task<Message> {
    window.generation += 1;
    window.responses = None;
    let result = window
        .backend
        .start(window.form.identity.uid, &window.form.request.cookie);
    let Conversation { responses, events } = match result {
        Ok(conversation) => conversation,
        Err(error) => {
            window.form.receive(Event::Failed(error));
            return Task::none();
        }
    };
    window.responses = Some(responses);
    let generation = window.generation;
    let stream = iced::futures::stream::unfold(events, |mut events| async {
        events.recv().await.map(|event| (event, events))
    });
    Task::run(stream, move |event| Message::Backend(generation, event))
}

pub fn update(window: &mut Window, message: Message) -> Task<Message> {
    match message {
        Message::Palette(palette) => window.skin = sparky_theme::Skin::new(palette),
        Message::Typed(response) => window.form.type_in(response),
        Message::Select(identity) if window.form.select(identity.uid) => return start(window),
        Message::Submit => return submit(window),
        Message::Next => return operation::focus_next(),
        Message::Close => return close(window, Outcome::Cancelled),
        Message::Backend(generation, event) if generation == window.generation => {
            let focus = matches!(event, Event::Prompt { .. });
            if let Some(outcome) = window.form.receive(event) {
                return close(window, outcome);
            }
            if focus {
                return operation::focus("polkit-response");
            }
        }
        _ => {}
    }
    Task::none()
}

fn submit(window: &mut Window) -> Task<Message> {
    if window.form.retry() {
        return start(window);
    }
    let Some(response) = window.form.submit() else {
        return Task::none();
    };
    let result = window
        .responses
        .as_ref()
        .ok_or("A autenticação não está ativa".to_owned())
        .and_then(|control| control.respond(response));
    if let Err(error) = result {
        window.form.receive(Event::Failed(error));
    }
    Task::none()
}

fn close(window: &mut Window, outcome: Outcome) -> Task<Message> {
    window.responses = None;
    window.outcome.store(outcome.code(), Ordering::SeqCst);
    iced::exit()
}
