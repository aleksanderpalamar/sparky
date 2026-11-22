use iced::{Task, window};

use super::message::Message;
use super::state::Dock;
use super::surfaces;
use crate::domain::layout::Geometry;

pub fn sync(dock: &mut Dock) -> Task<Message> {
    let wanted = dock.desired();
    match (dock.surface, wanted) {
        (None, None) => Task::none(),
        (Some(_), None) => {
            dock.surface = None;
            dock.hovering = false;
            dock.menu = None;
            Task::done(Message::RemoveWindow(dock.window))
        }
        (None, Some(geometry)) => {
            dock.surface = Some(geometry);
            dock.window = window::Id::unique();
            Task::done(Message::NewLayerShell {
                settings: surfaces::dock(geometry),
                id: dock.window,
            })
        }
        (Some(current), Some(geometry)) if current == geometry => Task::none(),
        (Some(current), Some(geometry)) => {
            dock.surface = Some(geometry);
            Task::batch(changes(dock.window, current, geometry))
        }
    }
}

fn changes(id: window::Id, current: Geometry, wanted: Geometry) -> Vec<Task<Message>> {
    let mut tasks = Vec::new();
    if current.zone != wanted.zone {
        tasks.push(Task::done(Message::ExclusiveZoneChange {
            id,
            zone_size: wanted.zone,
        }));
    }
    if current.size != wanted.size {
        tasks.push(Task::done(Message::SizeChange {
            id,
            size: wanted.size,
        }));
    }
    tasks
}
