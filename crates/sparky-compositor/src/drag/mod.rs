pub mod edges;
mod grab;

use smithay::input::pointer::{Focus, GrabStartData};
use smithay::utils::{Logical, Point, SERIAL_COUNTER, Serial};
use sparky_tiling::drag::Edges;
use sparky_tiling::drag_session::{Change, Drag, Gesture, Release};
use sparky_tiling::{Placement, Rect, WindowId};

use crate::drag::grab::WindowGrab;
use crate::state::Sparky;

pub const BUTTON_LEFT: u32 = 0x110;
pub const BUTTON_RIGHT: u32 = 0x111;

pub fn gesture(button: u32, frame: Rect, point: (i32, i32)) -> Option<Gesture> {
    match button {
        BUTTON_LEFT => Some(Gesture::Move),
        BUTTON_RIGHT => Some(Gesture::Resize(Edges::nearest(frame, point.0, point.1))),
        _ => None,
    }
}

pub fn point(location: Point<f64, Logical>) -> (i32, i32) {
    (location.x.round() as i32, location.y.round() as i32)
}

impl Sparky {
    pub fn start_super_drag(&mut self, button: u32, location: Point<f64, Logical>, serial: Serial) {
        if !self.super_held() {
            return;
        }
        let Some(window) = self.window_under(location) else {
            return;
        };
        let Some(placement) = self.placement_of(window) else {
            return;
        };
        if let Some(gesture) = gesture(button, placement.frame, point(location)) {
            self.begin_drag(window, gesture, button, location, serial);
        }
    }

    pub fn start_client_drag(&mut self, window: WindowId, gesture: Gesture, serial: Option<Serial>) {
        let Some(pointer) = self.seat.get_pointer() else {
            return;
        };
        if serial.is_some_and(|serial| !pointer.has_grab(serial)) {
            return;
        }
        let Some(start) = pointer.grab_start_data() else {
            return;
        };
        let serial = serial.unwrap_or_else(|| SERIAL_COUNTER.next_serial());
        self.begin_drag(window, gesture, start.button, start.location, serial);
    }

    pub fn follow_drag(&mut self, drag: &mut Drag, location: Point<f64, Logical>) {
        match drag.follow(point(location)) {
            Change::Frame(frame) => self.desktop.set_floating_frame(drag.window(), frame),
            Change::Grow(width, height) => self.desktop.resize_focused((width, height)),
            Change::Nothing => return,
        }
        self.relayout();
    }

    pub fn finish_drag(&mut self, drag: &Drag, location: Point<f64, Logical>) {
        if drag.release() != Release::SwapWithUnder {
            return;
        }
        let Some(target) = self.window_under(location) else {
            return;
        };
        if self.desktop.swap_windows(drag.window(), target) {
            self.relayout();
        }
    }

    fn begin_drag(&mut self, window: WindowId, gesture: Gesture, button: u32, location: Point<f64, Logical>, serial: Serial) {
        let (Some(pointer), Some(placement)) = (self.seat.get_pointer(), self.placement_of(window)) else {
            return;
        };
        let Some(drag) = Drag::start(window, gesture, placement.layer, placement.frame, point(location)) else {
            return;
        };
        self.desktop.focus_window(window);
        self.relayout();
        let start = GrabStartData { focus: None, button, location };
        pointer.set_grab(self, WindowGrab::new(start, drag), serial, Focus::Clear);
    }

    fn super_held(&self) -> bool {
        self.seat.get_keyboard().is_some_and(|keyboard| keyboard.modifier_state().logo)
    }

    fn placement_of(&self, window: WindowId) -> Option<Placement> {
        self.desktop
            .arrange()
            .into_iter()
            .flat_map(|(_, placements)| placements)
            .find(|placement| placement.window == window)
    }
}

#[cfg(test)]
mod tests {
    use super::{BUTTON_LEFT, BUTTON_RIGHT, gesture, point};
    use smithay::utils::Point;
    use sparky_tiling::Rect;
    use sparky_tiling::drag::{Edges, Horizontal, Vertical};
    use sparky_tiling::drag_session::Gesture;

    const CAIXA: Rect = Rect::new(0, 0, 400, 300);

    #[test]
    fn o_botao_esquerdo_move_e_o_direito_redimensiona_pelo_canto_mais_perto() {
        assert_eq!(gesture(BUTTON_LEFT, CAIXA, (10, 10)), Some(Gesture::Move));
        assert_eq!(
            gesture(BUTTON_RIGHT, CAIXA, (390, 20)),
            Some(Gesture::Resize(Edges { horizontal: Horizontal::Right, vertical: Vertical::Top }))
        );
    }

    #[test]
    fn o_botao_do_meio_nao_arrasta() {
        assert_eq!(gesture(0x112, CAIXA, (10, 10)), None);
    }

    #[test]
    fn o_ponto_do_ponteiro_e_arredondado() {
        assert_eq!(point(Point::from((10.6, -3.4))), (11, -3));
    }
}
