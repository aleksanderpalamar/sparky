use std::thread;
use std::time::Duration;

use iced::{Task, window};

use super::message::Message;
use super::state::Dock;
use crate::adapters::outputs;

const RETRY: Duration = Duration::from_secs(1);

pub fn closed(dock: &mut Dock, window: window::Id) -> Task<Message> {
    if dock
        .picker
        .as_ref()
        .is_some_and(|open| open.window == window)
    {
        dock.picker = None;
        return Task::none();
    }
    if window != dock.window || dock.surface.is_none() {
        return Task::none();
    }
    dock.surface = None;
    dock.accepting = None;
    dock.hovering = false;
    dock.menu = None;
    dock.drag = None;
    dock.displaced = true;
    super::update::measure()
}

pub fn found(dock: &mut Dock, width: Option<u32>) -> Task<Message> {
    dock.output = width;
    if !dock.displaced {
        return Task::none();
    }
    match width {
        Some(_) => {
            dock.displaced = false;
            Task::none()
        }
        None => retry(),
    }
}

fn retry() -> Task<Message> {
    Task::perform(
        async {
            tokio::task::spawn_blocking(|| {
                thread::sleep(RETRY);
                outputs::focused_width()
            })
            .await
        },
        |width| Message::Output(width.ok().flatten()),
    )
}
