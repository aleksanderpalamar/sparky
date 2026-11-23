use iced::Point;

use super::state::Dock;
use crate::domain::drag::{Drag, arranged, before, nearest};

pub fn moved(dock: &mut Dock, point: Point) {
    dock.pointer = Some(point);
    if let Some(drag) = dock.drag.as_mut() {
        drag.follow(point.x);
    }
}

pub fn pressed(dock: &mut Dock, id: &str) {
    dock.menu = None;
    let x = dock.pointer.map_or(0.0, |point| point.x);
    dock.drag = Some(Drag::new(id, x));
}

pub fn released(dock: &mut Dock) {
    let Some(drag) = dock.drag.take() else {
        return;
    };
    if !drag.moved {
        dock.start(&drag.id);
        return;
    }
    let order = visible(dock);
    let Some(target) = destination(dock, order.len()) else {
        return;
    };
    if let Some(next) = before(&order, &drag.id, target) {
        dock.place(&drag.id, next.as_deref());
    }
}

pub fn left(dock: &mut Dock) {
    dock.pointer = None;
    dock.drag = None;
}

pub fn order(dock: &Dock) -> Vec<String> {
    let order = visible(dock);
    let Some(drag) = dock.drag.as_ref().filter(|drag| drag.moved) else {
        return order;
    };
    destination(dock, order.len())
        .and_then(|target| arranged(&order, &drag.id, target))
        .unwrap_or(order)
}

fn visible(dock: &Dock) -> Vec<String> {
    dock.pinned
        .resolve(&dock.apps)
        .into_iter()
        .map(|app| app.id.clone())
        .collect()
}

fn destination(dock: &Dock, count: usize) -> Option<usize> {
    let x = dock.pointer?.x;
    let surface = dock.surface?.size.0 as f32;
    let centers = dock.layout().centers(count, surface);
    Some(nearest(x, centers.get(..count)?))
}
