use smithay::backend::input::{
    AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, PointerAxisEvent, PointerButtonEvent,
    PointerMotionEvent,
};
use smithay::input::pointer::{AxisFrame, ButtonEvent, MotionEvent};
use smithay::utils::{Logical, Point, SERIAL_COUNTER};

use crate::input::clamp::clamp;
use crate::layout::outputs::rect_of;
use crate::state::Sparky;

const WHEEL_STEP: f64 = 15.0 / 120.0;

impl Sparky {
    pub fn on_absolute_motion<I: InputBackend>(&mut self, event: I::PointerMotionAbsoluteEvent) {
        let Some(output) = self.space.outputs().next() else {
            return;
        };
        let Some(geometry) = self.space.output_geometry(output) else {
            return;
        };
        let position = event.position_transformed(geometry.size) + geometry.loc.to_f64();
        self.move_pointer(position, event.time_msec());
    }

    pub fn on_relative_motion<I: InputBackend>(&mut self, event: I::PointerMotionEvent) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        let moved = pointer.current_location() + event.delta();
        let screens: Vec<_> = self
            .space
            .outputs()
            .filter_map(|output| self.space.output_geometry(output))
            .map(rect_of)
            .collect();
        if let Some((x, y)) = clamp(moved.x, moved.y, &screens) {
            self.move_pointer((x, y).into(), event.time_msec());
        }
    }

    pub fn move_pointer(&mut self, position: Point<f64, Logical>, time: u32) {
        self.needs_render = true;
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        if !pointer.is_grabbed() {
            self.focus_follows_mouse(position);
        }
        let under = self.surface_under(position);
        let serial = SERIAL_COUNTER.next_serial();
        pointer.motion(self, under, &MotionEvent { location: position, serial, time });
        pointer.frame(self);
    }

    pub fn on_button<I: InputBackend>(&mut self, event: I::PointerButtonEvent) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        let serial = SERIAL_COUNTER.next_serial();
        if event.state() == ButtonState::Pressed {
            let location = pointer.current_location();
            self.focus_under(location);
            self.start_super_drag(event.button_code(), location, serial);
        }
        let button = ButtonEvent {
            button: event.button_code(),
            state: event.state(),
            serial,
            time: event.time_msec(),
        };
        pointer.button(self, &button);
        pointer.frame(self);
    }

    pub fn on_axis<I: InputBackend>(&mut self, event: I::PointerAxisEvent) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        let source = event.source();
        let mut frame = AxisFrame::new(event.time_msec()).source(source);
        for axis in [Axis::Horizontal, Axis::Vertical] {
            frame = scroll(frame, &event, axis, source);
        }
        pointer.axis(self, frame);
        pointer.frame(self);
    }

    fn focus_follows_mouse(&mut self, position: Point<f64, Logical>) {
        self.desktop.focus_monitor_at(position.x as i32, position.y as i32);
        if let Some(id) = self.window_under(position)
            && self.desktop.focused_window() != Some(id)
        {
            self.desktop.focus_window(id);
            self.relayout();
        }
    }

    fn focus_under(&mut self, position: Point<f64, Logical>) {
        if let Some(id) = self.window_under(position) {
            self.desktop.focus_window(id);
            self.relayout();
        }
    }
}

fn scroll<E: PointerAxisEvent<I>, I: InputBackend>(frame: AxisFrame, event: &E, axis: Axis, source: AxisSource) -> AxisFrame {
    let discrete = event.amount_v120(axis);
    let amount = event
        .amount(axis)
        .unwrap_or_else(|| discrete.unwrap_or(0.0) * WHEEL_STEP);
    let mut frame = frame;
    if amount != 0.0 {
        frame = frame.value(axis, amount);
        if let Some(discrete) = discrete {
            frame = frame.v120(axis, discrete as i32);
        }
    }
    if source == AxisSource::Finger && event.amount(axis) == Some(0.0) {
        frame = frame.stop(axis);
    }
    frame
}
