use iced::{Task, window};
use iced_layershell::actions::ActionCallback;

use super::glass;
use super::message::Message;
use super::state::Dock;
use super::surfaces;
use crate::domain::layout::Geometry;

pub fn sync(dock: &mut Dock) -> Task<Message> {
    let shape = reshape(dock);
    glaze(dock);
    Task::batch([shape, accept(dock)])
}

fn glaze(dock: &mut Dock) {
    let Some(frame) = dock.surface else {
        return;
    };
    let count = dock.pinned.resolve(&dock.apps).len();
    let mut scales = super::animate::scales(dock);
    scales.resize(count + 1, 1.0);
    let pose = glass::Pose {
        scales,
        sink: dock.slide.value * dock.layout().rest() as f32,
        tucked: dock.tucked(),
    };
    let wanted = glass::area(&dock.layout(), &pose, frame, dock.preferences.corner());
    if dock.glass.as_ref() != Some(&wanted) {
        dock.glass = Some(wanted.clone());
        dock.show_glass(wanted);
    }
}

fn accept(dock: &mut Dock) -> Task<Message> {
    let wanted = dock.region();
    if wanted == dock.accepting {
        return Task::none();
    }
    dock.accepting = wanted;
    let Some(rect) = wanted else {
        return Task::none();
    };
    Task::done(Message::SetInputRegion {
        id: dock.window,
        callback: ActionCallback::new(move |region| {
            region.add(rect.x, rect.y, rect.width, rect.height);
        }),
    })
}

fn reshape(dock: &mut Dock) -> Task<Message> {
    let wanted = dock.desired();
    match (dock.surface, wanted) {
        (None, None) => Task::none(),
        (Some(_), None) => {
            dock.surface = None;
            dock.accepting = None;
            dock.hovering = false;
            dock.menu = None;
            Task::done(Message::RemoveWindow(dock.window))
        }
        (None, Some(geometry)) => {
            dock.surface = Some(geometry);
            dock.accepting = None;
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
