use smithay::input::pointer::{CursorIcon, CursorImageStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostCursor {
    Shape(CursorIcon),
    Hidden,
}

pub fn host_cursor(status: &CursorImageStatus) -> HostCursor {
    match status {
        CursorImageStatus::Named(icon) => HostCursor::Shape(*icon),
        CursorImageStatus::Hidden => HostCursor::Hidden,
        CursorImageStatus::Surface(_) => HostCursor::Shape(CursorIcon::Default),
    }
}

#[cfg(test)]
mod tests {
    use super::{HostCursor, host_cursor};
    use smithay::input::pointer::{CursorIcon, CursorImageStatus};

    #[test]
    fn o_formato_pedido_vai_para_o_cursor_do_host() {
        assert_eq!(host_cursor(&CursorImageStatus::Named(CursorIcon::Text)), HostCursor::Shape(CursorIcon::Text));
    }

    #[test]
    fn cursor_escondido_esconde_o_do_host() {
        assert_eq!(host_cursor(&CursorImageStatus::Hidden), HostCursor::Hidden);
    }
}
