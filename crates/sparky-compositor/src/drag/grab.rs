use smithay::input::pointer::{
    AxisFrame, ButtonEvent, GestureHoldBeginEvent, GestureHoldEndEvent, GesturePinchBeginEvent, GesturePinchEndEvent,
    GesturePinchUpdateEvent, GestureSwipeBeginEvent, GestureSwipeEndEvent, GestureSwipeUpdateEvent, GrabStartData,
    MotionEvent, PointerGrab, PointerInnerHandle, RelativeMotionEvent,
};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point};
use sparky_tiling::drag_session::Drag;

use crate::state::Sparky;

type Handle<'a> = PointerInnerHandle<'a, Sparky>;
type Under = Option<(WlSurface, Point<f64, Logical>)>;

pub struct WindowGrab {
    start: GrabStartData<Sparky>,
    drag: Drag,
}

impl WindowGrab {
    pub fn new(start: GrabStartData<Sparky>, drag: Drag) -> Self {
        Self { start, drag }
    }
}

impl PointerGrab<Sparky> for WindowGrab {
    fn motion(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, _focus: Under, event: &MotionEvent) {
        handle.motion(data, None, event);
        data.follow_drag(&mut self.drag, event.location);
    }

    fn relative_motion(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, _focus: Under, event: &RelativeMotionEvent) {
        handle.relative_motion(data, None, event);
    }

    fn button(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, event: &ButtonEvent) {
        handle.button(data, event);
        if handle.current_pressed().is_empty() {
            data.finish_drag(&self.drag, handle.current_location());
            handle.unset_grab(self, data, event.serial, event.time, true);
        }
    }

    fn axis(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, details: AxisFrame) {
        handle.axis(data, details);
    }

    fn frame(&mut self, data: &mut Sparky, handle: &mut Handle<'_>) {
        handle.frame(data);
    }

    fn gesture_swipe_begin(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, event: &GestureSwipeBeginEvent) {
        handle.gesture_swipe_begin(data, event);
    }

    fn gesture_swipe_update(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, event: &GestureSwipeUpdateEvent) {
        handle.gesture_swipe_update(data, event);
    }

    fn gesture_swipe_end(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, event: &GestureSwipeEndEvent) {
        handle.gesture_swipe_end(data, event);
    }

    fn gesture_pinch_begin(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, event: &GesturePinchBeginEvent) {
        handle.gesture_pinch_begin(data, event);
    }

    fn gesture_pinch_update(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, event: &GesturePinchUpdateEvent) {
        handle.gesture_pinch_update(data, event);
    }

    fn gesture_pinch_end(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, event: &GesturePinchEndEvent) {
        handle.gesture_pinch_end(data, event);
    }

    fn gesture_hold_begin(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, event: &GestureHoldBeginEvent) {
        handle.gesture_hold_begin(data, event);
    }

    fn gesture_hold_end(&mut self, data: &mut Sparky, handle: &mut Handle<'_>, event: &GestureHoldEndEvent) {
        handle.gesture_hold_end(data, event);
    }

    fn start_data(&self) -> &GrabStartData<Sparky> {
        &self.start
    }

    fn unset(&mut self, _data: &mut Sparky) {}
}
