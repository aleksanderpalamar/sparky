use std::time::Instant;

use super::state::Dock;
use crate::domain::layout::Visibility;
use crate::domain::magnify::targets;
use crate::domain::motion::Ease;

const ZOOM_PACE: f32 = 0.06;
const SLIDE_PACE: f32 = 0.08;
const LONGEST_FRAME: f32 = 0.1;

pub fn aim(dock: &mut Dock) {
    let hidden = dock.visibility() == Visibility::Hidden;
    dock.slide.aim(if hidden { 1.0 } else { 0.0 });
    let count = dock.pinned.resolve(&dock.apps).len();
    let still = hidden || dock.menu.is_some() || dock.drag.as_ref().is_some_and(|drag| drag.moved);
    let pointer = dock.pointer.filter(|_| !still).map(|point| point.x);
    let layout = dock.layout();
    let surface = dock.surface.map_or(0.0, |geometry| geometry.size.0 as f32);
    let wanted = targets(
        pointer,
        &layout.centers(count, surface),
        layout.slot + layout.gap(),
    );
    dock.zoom.resize(wanted.len(), Ease::at(1.0));
    for (ease, target) in dock.zoom.iter_mut().zip(wanted) {
        ease.aim(target);
    }
}

pub fn animating(dock: &Dock) -> bool {
    !dock.slide.settled() || dock.zoom.iter().any(|ease| !ease.settled())
}

pub fn advance(dock: &mut Dock, now: Instant) {
    let elapsed = dock
        .clock
        .map_or(0.0, |last| now.duration_since(last).as_secs_f32())
        .min(LONGEST_FRAME);
    dock.slide.step(elapsed, SLIDE_PACE);
    for ease in &mut dock.zoom {
        ease.step(elapsed, ZOOM_PACE);
    }
    dock.clock = animating(dock).then_some(now);
}

pub fn scales(dock: &Dock) -> Vec<f32> {
    dock.zoom.iter().map(|ease| ease.value).collect()
}
