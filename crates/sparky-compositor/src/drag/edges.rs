use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::xwayland::xwm::ResizeEdge;
use sparky_tiling::drag::{Edges, Horizontal, Vertical};

pub fn from_xdg(edge: xdg_toplevel::ResizeEdge) -> Edges {
    let (horizontal, vertical) = match edge {
        xdg_toplevel::ResizeEdge::Top => (Horizontal::Neither, Vertical::Top),
        xdg_toplevel::ResizeEdge::Bottom => (Horizontal::Neither, Vertical::Bottom),
        xdg_toplevel::ResizeEdge::Left => (Horizontal::Left, Vertical::Neither),
        xdg_toplevel::ResizeEdge::Right => (Horizontal::Right, Vertical::Neither),
        xdg_toplevel::ResizeEdge::TopLeft => (Horizontal::Left, Vertical::Top),
        xdg_toplevel::ResizeEdge::TopRight => (Horizontal::Right, Vertical::Top),
        xdg_toplevel::ResizeEdge::BottomLeft => (Horizontal::Left, Vertical::Bottom),
        xdg_toplevel::ResizeEdge::BottomRight => (Horizontal::Right, Vertical::Bottom),
        _ => (Horizontal::Neither, Vertical::Neither),
    };
    Edges { horizontal, vertical }
}

pub fn from_x11(edge: ResizeEdge) -> Edges {
    let (horizontal, vertical) = match edge {
        ResizeEdge::Top => (Horizontal::Neither, Vertical::Top),
        ResizeEdge::Bottom => (Horizontal::Neither, Vertical::Bottom),
        ResizeEdge::Left => (Horizontal::Left, Vertical::Neither),
        ResizeEdge::Right => (Horizontal::Right, Vertical::Neither),
        ResizeEdge::TopLeft => (Horizontal::Left, Vertical::Top),
        ResizeEdge::TopRight => (Horizontal::Right, Vertical::Top),
        ResizeEdge::BottomLeft => (Horizontal::Left, Vertical::Bottom),
        ResizeEdge::BottomRight => (Horizontal::Right, Vertical::Bottom),
    };
    Edges { horizontal, vertical }
}

#[cfg(test)]
mod tests {
    use super::{from_x11, from_xdg};
    use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
    use smithay::xwayland::xwm::ResizeEdge;
    use sparky_tiling::drag::{Edges, Horizontal, Vertical};

    #[test]
    fn as_bordas_do_xdg_viram_os_lados_que_esticam() {
        assert_eq!(
            from_xdg(xdg_toplevel::ResizeEdge::TopLeft),
            Edges { horizontal: Horizontal::Left, vertical: Vertical::Top }
        );
        assert_eq!(
            from_xdg(xdg_toplevel::ResizeEdge::Right),
            Edges { horizontal: Horizontal::Right, vertical: Vertical::Neither }
        );
        assert_eq!(
            from_xdg(xdg_toplevel::ResizeEdge::None),
            Edges { horizontal: Horizontal::Neither, vertical: Vertical::Neither }
        );
    }

    #[test]
    fn as_bordas_do_x11_valem_o_mesmo() {
        assert_eq!(
            from_x11(ResizeEdge::BottomRight),
            Edges { horizontal: Horizontal::Right, vertical: Vertical::Bottom }
        );
        assert_eq!(from_x11(ResizeEdge::Top), Edges { horizontal: Horizontal::Neither, vertical: Vertical::Top });
    }
}
