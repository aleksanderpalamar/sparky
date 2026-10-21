use smithay::backend::input::{Event, InputBackend, PointerMotionEvent};
use smithay::input::pointer::{PointerHandle, RelativeMotionEvent};
use smithay::utils::{Logical, Point};
use smithay::wayland::pointer_constraints::{PointerConstraint, with_pointer_constraint};

use crate::input::clamp::clamp;
use crate::input::hit::Under;
use crate::input::restriction::{Landing, Restriction, Step, step};
use crate::layout::outputs::rect_of;
use crate::state::Sparky;

impl Sparky {
    pub fn on_relative_motion<I: InputBackend>(&mut self, event: I::PointerMotionEvent) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        let current = pointer.current_location();
        let under = self.surface_under(current);
        let relative = RelativeMotionEvent {
            delta: event.delta(),
            delta_unaccel: event.delta_unaccel(),
            utime: event.time(),
        };
        pointer.relative_motion(self, under.clone(), &relative);
        let restriction = restriction(&pointer, under.as_ref(), current);
        let target = self.on_screen(current + event.delta());
        let landing = target.map_or(Landing::Outside, |target| self.landing(&pointer, under.as_ref(), target));
        match (step(restriction, landing), target) {
            (Step::Move, Some(target)) => {
                self.move_pointer(target, event.time_msec());
                self.activate_constraint(&pointer, target);
            }
            _ => pointer.frame(self),
        }
    }

    fn on_screen(&self, wanted: Point<f64, Logical>) -> Option<Point<f64, Logical>> {
        let screens: Vec<_> = self
            .space
            .outputs()
            .filter_map(|output| self.space.output_geometry(output))
            .map(rect_of)
            .collect();
        clamp(wanted.x, wanted.y, &screens).map(Point::from)
    }

    fn landing(&self, pointer: &PointerHandle<Sparky>, under: Option<&Under>, target: Point<f64, Logical>) -> Landing {
        let Some((surface, origin)) = under else {
            return Landing::Outside;
        };
        let same = self.surface_under(target).is_some_and(|(found, _)| &found == surface);
        let region = with_pointer_constraint(surface, pointer, |constraint| constraint.and_then(|constraint| constraint.region().cloned()));
        let inside = region.is_none_or(|region| region.contains((target - *origin).to_i32_round()));
        match same && inside {
            true => Landing::Inside,
            false => Landing::Outside,
        }
    }

    fn activate_constraint(&mut self, pointer: &PointerHandle<Sparky>, location: Point<f64, Logical>) {
        let Some((surface, origin)) = self.surface_under(location) else {
            return;
        };
        with_pointer_constraint(&surface, pointer, |constraint| {
            let Some(constraint) = constraint.filter(|constraint| !constraint.is_active()) else {
                return;
            };
            if constraint.region().is_none_or(|region| region.contains((location - origin).to_i32_round())) {
                constraint.activate();
            }
        });
    }
}

fn restriction(pointer: &PointerHandle<Sparky>, under: Option<&Under>, location: Point<f64, Logical>) -> Restriction {
    let Some((surface, origin)) = under else {
        return Restriction::Free;
    };
    with_pointer_constraint(surface, pointer, |constraint| {
        let Some(constraint) = constraint.filter(|constraint| constraint.is_active()) else {
            return Restriction::Free;
        };
        if !constraint.region().is_none_or(|region| region.contains((location - *origin).to_i32_round())) {
            return Restriction::Free;
        }
        match &*constraint {
            PointerConstraint::Locked(_) => Restriction::Locked,
            PointerConstraint::Confined(_) => Restriction::Confined,
        }
    })
}
